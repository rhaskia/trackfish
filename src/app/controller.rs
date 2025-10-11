use super::{
    playlist::get_playlist_files,
    playlist::Playlist,
    queue::{Listen, Queue, QueueType},
    settings::{RadioSettings, Settings, WeightMode},
    track::{Mood, Track, TrackInfo},
    utils::{similar, strip_unnessecary}, autoplaylist::AutoPlaylist,
};
use crate::analysis::{generate_track_info, utils::cosine_similarity};
use crate::database::{hash_filename, save_track_weights};
use log::{info, warn, error};
use ndarray::Array1;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::thread_rng;
use rusqlite::Connection;
use rustfft::num_traits::Zero;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static MUSIC_PLAYER_ACTIONS: Lazy<Mutex<Option<Sender<MusicMsg>>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(Debug)]
pub enum MusicMsg {
    Skip,
    SkipBack,
    Pause,
    Play,
    Toggle,
    PlayTrack(String),
    SetVolume(f32),
    SetPos(f64),
}

// Send message to AudioPlayer in thread
pub fn send_music_msg(msg: MusicMsg) {
    if let Some(tx) = MUSIC_PLAYER_ACTIONS.lock().unwrap().as_ref() {
        if let Err(e) = tx.send(msg) {
            info!("send error: {e:?}");
        }
    } else {
        info!("no MUSIC_PLAYER_ACTIONS set");
    }
}

#[derive(PartialEq, Clone)]
pub struct MusicController {
    pub all_tracks: Vec<Track>,
    pub track_info: Vec<TrackInfo>,
    pub artists: HashMap<String, (String, usize)>,
    pub genres: HashMap<String, usize>,
    pub albums: HashMap<String, usize>,
    pub listens: Vec<Listen>,
    pub shuffle: bool,
    pub playlists: Vec<Playlist>,
    pub autoplaylists: Vec<AutoPlaylist>,
    current_started: Instant,

    pub current_queue: usize,
    pub queues: Vec<Queue>,
    pub settings: Settings,
    pub progress_secs: f64,
    pub song_length: f64,
    pub playing: bool,
}

// Basic functionality
impl MusicController {
    /// Creates an empty controller with no tracks
    pub fn empty() -> Self {
        Self {
            all_tracks: vec![],
            track_info: vec![],
            artists: HashMap::new(),
            genres: HashMap::new(),
            albums: HashMap::new(),
            listens: vec![],
            current_started: Instant::now(),
            current_queue: 0,
            queues: vec![Queue::all()],
            settings: Settings::load(),
            shuffle: false,
            playlists: Vec::new(),
            autoplaylists: Vec::new(),
            progress_secs: 0.0,
            song_length: 100.0,
            playing: false,
        }
    }

    /// Creates and loads all tracks and weights into the controller
    pub fn new(all_tracks: Vec<Track>, _directory: String) -> Self {
        let mut rng = thread_rng();
        let current_playing = if all_tracks.len() > 0 {
            rng.gen_range(0..all_tracks.len())
        } else {
            0
        };

        let started = std::time::SystemTime::now();

        let (mut albums, mut artists, mut genres) =
            (HashMap::new(), HashMap::new(), HashMap::new());

        for track in &all_tracks {
            for genre in track.genres.clone() {
                *genres.entry(genre.clone()).or_insert(0) += 1;
            }

            for artist in track.artists.clone() {
                // some artists names seem to change captalization grr
                let stripped = strip_unnessecary(&artist);
                artists.entry(stripped).or_insert((artist, 0)).1 += 1;
            }

            *albums.entry(track.album.clone()).or_insert(0) += 1;
        }

        let mut controller = MusicController {
            all_tracks: all_tracks.clone(),
            current_started: Instant::now(),
            listens: Vec::new(),
            queues: vec![Queue::radio(
                current_playing,
                all_tracks
                    .get(current_playing)
                    .cloned()
                    .unwrap_or_default()
                    .title,
            )],
            current_queue: 0,
            track_info: Vec::new(),
            artists,
            genres,
            albums,
            settings: Settings::load(),
            shuffle: false,
            playlists: Vec::new(),
            autoplaylists: Vec::new(),
            progress_secs: 0.0,
            song_length: 100.0,
            playing: true,
        };

        send_music_msg(MusicMsg::SetVolume(controller.settings.volume));

        controller.load_playlists();
        controller.load_autoplaylists();

        info!("Calculated weights in {:?}", started.elapsed());

        if let Some(track) = controller.current_track().cloned() {
            send_music_msg(MusicMsg::PlayTrack(track.file.clone()));
            controller.toggle_playing();
            info!("Started track {track:?}");
        }

        controller
    }

    /// Loads all playlists in the music directory (.m3u files)
    pub fn load_playlists(&mut self) {
        let files = get_playlist_files(&self.settings.directory).unwrap();

        for file in files {
            let playlist = Playlist::load(&self.settings.directory, &file, &self.all_tracks);
            self.playlists.push(playlist);
        }
    }

    /// Loads all autoplaylists saved in cache (.auto files)
    pub fn load_autoplaylists(&mut self) {
        for entry in std::fs::read_dir(Settings::dir()).unwrap() {
            let path = entry.unwrap().path();
            let filename = path.to_str().unwrap().to_string();

            if path.is_file() {
                if path.extension().unwrap_or_default().to_str().unwrap_or_default() == "auto" {
                    match AutoPlaylist::load(path) {
                        Ok(ap) => self.autoplaylists.push(ap),
                        Err(e) => error!("{e:?}"),
                    }
                }
            }
        }
    }

    /// Deletes an autoplaylist from storage and memory
    pub fn rename_autoplaylist(&mut self, autoplaylist: usize, name: String) {
        let path = self.autoplaylists[autoplaylist].dir();
        std::fs::remove_file(path).unwrap();
        self.autoplaylists[autoplaylist].name = name;
        self.autoplaylists[autoplaylist].save();
    }

    /// Deletes an autoplaylist from storage and memory
    pub fn delete_autoplaylist(&mut self, autoplaylist: usize) {
        let path = self.autoplaylists[autoplaylist].dir();
        std::fs::remove_file(path).unwrap();
        self.autoplaylists.remove(autoplaylist);
    }

    /// Deletes a playlist from storage and memory
    pub fn delete_playlist(&mut self, playlist: usize) {
        let path = self.playlists[playlist].file.clone();
        std::fs::remove_file(path).unwrap();
        self.playlists.remove(playlist);
    }

    /// Saves a playlist in the M3U format
    pub fn save_playlist(&mut self, playlist: usize) {
        let playlist = self.playlists[playlist].clone();
        let relative_paths: Vec<String> = playlist
            .tracks
            .iter()
            .map(|t| relative_path(&self.all_tracks[*t].file, &self.settings.directory))
            .collect();

        let file = String::from("#EXTM3U\n#PLAYLIST:")
            + &playlist.name
            + "\n"
            + &relative_paths.join("\n\n");
        std::fs::write(&playlist.file, file).unwrap();
    }

    /// Loads a weight, either using the database or by calculating them 
    pub fn load_weight(
        &mut self,
        cache: &Connection,
        weights: &HashMap<String, TrackInfo>,
        track_idx: usize,
    ) -> bool {
        let track = &self.all_tracks[track_idx];
        let file_hash = hash_filename(&track.file);
        if weights.contains_key(&file_hash) {
            self.track_info.push(weights[&file_hash].clone());
            return true;
        } else {
            let track_info = generate_track_info(&track);
            save_track_weights(&cache, &track.file, &track_info).unwrap();
            self.track_info.push(track_info);
            return false;
        }
    }

    /// Plays a given track
    pub fn play_track(&mut self, idx: usize) {
        if let Some(current_track) = self.current_track() {
            self.listens.push(Listen::new(
                self.current_track_idx(),
                self.current_started,
                current_track.len,
                self.progress_secs,
            ));
        }

        self.current_started = Instant::now();
        self.progress_secs = 0.0;

        send_music_msg(MusicMsg::PlayTrack(self.all_tracks[idx].file.clone()))
    }

    /// Returns the current track weights, or average track weights accross the queue
    pub fn get_space(&mut self) -> TrackInfo {
        match self.settings.radio.weight_mode {
            WeightMode::First => self.track_info[self.current_queue().cached_order[0]].clone(),
            WeightMode::Last => {
                self.track_info[*self.current_queue().cached_order.iter().last().unwrap()].clone()
            }
            WeightMode::Average => {
                let mut tracks = Vec::new();

                // Introduce count later?
                let count = self.current_queue().cached_order.len();
                for i in (count.max(10) - 10)..count {
                    tracks.push(
                        self.track_info
                            .get(self.current_queue().cached_order[i])
                            .cloned()
                            .unwrap_or_default(),
                    );
                }

                TrackInfo::average(tracks)
            }
        }
    }

    /// Returns all given weights for tracks in the player
    pub fn get_weights(&mut self) -> Array1<f32> {
        let space = self.get_space();

        let mut weights = Array1::from_vec(vec![0.0; self.all_tracks.len()]);
        let mut dists: Vec<(usize, f32)> = self
            .track_info
            .iter()
            .map(|track| genres_dist_from_vec(&track, &space, &self.settings.radio))
            .enumerate()
            .collect();
        dists.sort_by(|(_, a), (_, b)| b.total_cmp(a));

        let mut count = 0;
        let amount = 20;
        let temperature = 10.0;
        for (rank, (song, distance)) in dists.iter().enumerate() {
            if count == 50 {
                break;
            }
            if rank == 0 {
                continue;
            }
            if self.current_queue().cached_order.contains(song) {
                continue;
            }

            let norm = (amount - count) as f32 / amount as f32;
            weights[*song] = 1.0 / ((norm * temperature - (temperature / 3.0)).exp() + 1.05) + 0.05;
            if weights[*song].is_sign_negative() {
                info!("{song}, {rank}, {distance}");
            }
            count += 1;
        }

        for weight in &mut weights {
            if weight.is_nan() || weight.is_sign_negative() {
                *weight = 0.0;
                info!("NaN weight found");
            }
        }

        for i in 0..self.all_tracks.len() {
            let current_idx = self.current_queue().current();
            if similar(
                &self.all_tracks[current_idx].album,
                &self.all_tracks[i].album,
            ) {
                weights *= self.settings.radio.album_penalty;
            }

            if self.all_tracks[current_idx].shared_artists(&self.all_tracks[i]) > 0 {
                weights *= self.settings.radio.artist_penalty;
            }
        }

        // TODO: weights for each feature used in weighting

        weights
    }

    /// Returns the next 'similar' track to play
    pub fn next_similar(&mut self) -> usize {
        log::info!("next");
        let mut weights = self.get_weights().to_vec();
        if weights.iter().all(|w| w.is_zero()) {
            warn!("All weights zero");
            weights = vec![1.0; weights.len()];
        }
        let dist = WeightedIndex::new(weights.clone()).unwrap();
        let mut rng = thread_rng();

        let next = dist.sample(&mut rng);
        info!("chosen weight {}", weights[next]);
        next
    }

    /// Skips to the previous song in queue
    pub fn skipback(&mut self) {
        if self.progress_secs < 5.0 {
            if self.queues[self.current_queue].current_track == 0 {
                return;
            }
            let last = self.queues[self.current_queue].current_track - 1;
            self.queues[self.current_queue].current_track = last;
        }

        self.play_track(self.queues[self.current_queue].current());
    }

    /// Skips the current track in the queue, or skips to the next queue if at end of queue
    pub fn skip(&mut self) {
        if self.all_tracks.is_empty() {
            log::info!("No track to skip to");
            return;
        }

        let current_queue = &mut self.queues[self.current_queue];

        // next track exists in queue
        if let Some(next) = current_queue
            .cached_order
            .get(current_queue.current_track + 1)
            .cloned()
        {
            current_queue.current_track += 1;
            self.play_track(next);
            return;
        }

        match current_queue.queue_type {
            QueueType::Radio(_) => {
                let next = self.next_similar();
                self.queues[self.current_queue].current_track += 1;
                self.queues[self.current_queue].cached_order.push(next);
                self.play_track(next);
            }
            _ => {
                if self.queues.len() > self.current_queue + 1 {
                    self.current_queue += 1;
                    // TODO: shuffle next queue if needed
                    self.play_track(self.current_queue().track(0))
                }
            }
        }
    }

    /// Sets the current queue playing and at which track
    pub fn set_queue_and_track(&mut self, queue: usize, track: usize) {
        self.current_queue = queue;
        self.queues[queue].current_track = track;
        self.play_track(self.queues[queue].cached_order[track]);
    }

    /// Returns tracks matching a certain QueueType
    pub fn get_matching(&self, queue_type: QueueType) -> Vec<usize> {
        if queue_type == QueueType::AllTracks {
            return (0..self.all_tracks.len()).collect();
        }

        self.all_tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| track.matches(queue_type.clone()))
            .map(|(index, _)| index)
            .collect()
    }

    /// Removes a queue from the queue list and moves to another queue
    /// TODO: some better way of choosing the queue to shift to 
    pub fn remove_queue(&mut self, queue: usize) {
        if self.current_queue == queue && self.current_queue != 0 {
            self.current_queue -= 1;
        }
        self.queues.remove(queue);
    }

    /// Creates a playlist using tracks in a given queue
    pub fn queue_to_playlist(&mut self, queue: usize) {
        let queue = self.queues[queue].clone();
        let mut playlist = Playlist::new(
            format!("{}", queue.queue_type),
            self.settings.directory.clone(),
        );
        playlist.tracks = queue.cached_order;
        self.playlists.push(playlist);
        self.save_playlist(self.playlists.len() - 1);

        // TODO replace queue with playlist queue?
    }

    /// Adds a list of tracks to a given queue
    pub fn add_tracks_to_queue(&mut self, queue: usize, tracks: Vec<usize>) {
        self.queues[queue].cached_order.extend(tracks);
    }

    /// Adds a list of tracks to a given playlist
    pub fn add_tracks_to_playlist(&mut self, playlist: usize, tracks: Vec<usize>) {
        self.playlists[playlist].tracks.extend(tracks);
    }
}

// Queue creation
impl MusicController {
    /// Starts an artist queue at no specific starting track
    pub fn add_artist_queue(&mut self, artist: String) {
        let tracks = self.get_tracks_where(|track| track.artists.contains(&artist));
        self.queues
            .push(Queue::new(QueueType::Artist(artist), tracks));
        self.current_queue = self.queues.len() - 1;
    }
    
    /// Starts an album queue starting with a specified track
    pub fn play_album_at(&mut self, album: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.album == album);
        self.add_queue_at(tracks, QueueType::Album(album.clone()), track);
    }

    /// Starts an genre queue starting with a specified track
    pub fn play_genre_at(&mut self, genre: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.has_genre(&genre));
        self.add_queue_at(tracks, QueueType::Genre(genre.clone()), track);
    }

    /// Starts an artist queue starting with a specified track
    pub fn play_artist_at(&mut self, artist: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.has_artist(&artist));
        self.add_queue_at(tracks, QueueType::Artist(artist.clone()), track);
    }

    /// Starts a radio queue with a specified starting track
    pub fn start_radio(&mut self, track: usize) {
        let track_name = self.all_tracks[track].title.clone();
        self.add_queue_at(vec![track], QueueType::Radio(track_name), track);
    }

    /// Starts a playlist, with a given track to start
    pub fn play_playlist_at(&mut self, playlist: usize, track: usize) {
        self.add_queue_at(
            self.playlists[playlist].tracks.clone(),
            QueueType::Playlist(self.playlists[playlist].name.clone(), playlist),
            track,
        );
    }

    /// Starts an autoplaylist, with a given track to start
    pub fn play_autoplaylist_at(&mut self, tracks: Vec<usize>, autoplaylist: usize, track: usize) {
        self.add_queue_at(
            tracks,
            QueueType::AutoPlaylist(self.autoplaylists[autoplaylist].name.clone(), autoplaylist),
            track,
        );
    }

    /// Starts a given queue with some tracks at a specific track
    pub fn add_queue_at(&mut self, mut tracks: Vec<usize>, queue: QueueType, track: usize) {
        if self.shuffle {
            tracks = shuffle_with_first(tracks, track);
        }

        info!("{track}");
        let track_idx = tracks.iter().position(|e| *e == track).unwrap();

        for i in 0..self.queues.len() {
            if self.queues[i].queue_type == queue {
                self.queues[i].cached_order = tracks;
                self.queues[i].current_track = track_idx;
                self.current_queue = i;
                self.play_track(track);
                self.play();
                info!("{}, {}", self.queues[i].current(), track);
                return;
            }
        }

        self.queues.push(Queue::new(queue, tracks));
        self.current_queue = self.queues.len() - 1;
        self.queues[self.current_queue].current_track = track_idx;
        self.play_track(track);
        self.play();
    }

    /// Add a queue containing all tracks, with a given track to start
    pub fn add_all_queue(&mut self, track: usize) {
        let tracks = (0..self.all_tracks.len()).collect();
        self.add_queue_at(tracks, QueueType::AllTracks, track);
    }

    /// Get tracks that fit a given conditional, using a supplied Fn
    pub fn get_tracks_where<F>(&self, condition: F) -> Vec<usize>
    where
        F: Fn(&Track) -> bool,
    {
        self.all_tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| condition(*track))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Toggles between shuffled and unshuffled in all queues
    pub fn toggle_shuffle(&mut self) {
        if self.shuffle {
            // unshuffle queues
            for queue in &mut self.queues {
                let current = queue.cached_order[queue.current_track];

                match queue.queue_type {
                    QueueType::Radio(_) => {}
                    QueueType::Album(_) => queue.cached_order.sort_by(|a, b| {
                        self.all_tracks[*a]
                            .trackno
                            .cmp(&self.all_tracks[*b].trackno)
                    }),
                    _ => queue.cached_order.sort_by(|a, b| a.cmp(b)),
                }

                // Keep same track playing
                let new_idx = queue.cached_order.iter().position(|n| *n == current);
                queue.current_track = new_idx.unwrap_or(0);
            }
        } else {
            for queue in &mut self.queues {
                if let QueueType::Radio(_) = queue.queue_type {
                    // Painful to try and unshuffle radio queues
                    continue;
                }

                queue.cached_order =
                    shuffle_with_first(queue.cached_order.clone(), queue.current());
                queue.current_track = 0;
            }
        }

        self.shuffle = !self.shuffle
    }

    /// Adds a track to the spot after the current track in queue
    pub fn play_next(&mut self, track: usize) {
        let position = self.current_queue().current_track;
        self.mut_current_queue()
            .cached_order
            .insert(position + 1, track);
    }

    /// Adds a track to a given playlist
    pub fn add_to_playlist(&mut self, playlist: usize, track: usize) {
        let file = relative_path(&self.all_tracks[track].file, &self.settings.directory);
        info!("{file}");
        self.playlists[playlist].tracks.push(track);
        self.playlists[playlist].track_paths.push(file);
        self.save_playlist(playlist);
    }
}

/// Returns the part of two paths that they do not share
/// Used to get part of a music file path without the initial music directory path
/// Almost definitely flawed in the way it is coded
pub fn relative_path(file: &str, dir: &str) -> String {
    let file_canon = PathBuf::from(file).canonicalize().unwrap();
    let dir_canon = PathBuf::from(dir).canonicalize().unwrap();
    let mut file_comp = file_canon.components();
    let mut dir_comp = dir_canon.components();

    while dir_comp.next().is_some() {
        file_comp.next();
    }

    file_comp.as_path().to_string_lossy().to_string()
}

/// Shuffles a list while keeping an item at the start
/// Used so that the shuffle button does not immediately play a new track
pub fn shuffle_with_first(mut tracks: Vec<usize>, start: usize) -> Vec<usize> {
    if let Some(idx) = tracks.iter().position(|e| *e == start) {
        tracks.remove(idx);
    }

    // Probably could use a nicer shuffle method later on
    let mut rng = thread_rng();
    tracks.shuffle(&mut rng);

    tracks.insert(0, start);

    tracks
}

// Settings Management
impl MusicController {
    /// Sets the volume of the music player and saves it to storage
    pub fn set_volume(&mut self, volume: f32) {
        self.settings.volume = volume;
        send_music_msg(MusicMsg::SetVolume(volume));
        self.settings.save();
        info!("Set volume to {volume}");
    }

    /// Sets the music directory, and saves it to storage
    pub fn set_directory(&mut self, new_dir: String) {
        self.settings.directory = new_dir;
        self.settings.save();
        // Manage loading new tracks
    }

    /// Sets the 'temperature' of the reccomendation system
    pub fn set_temp(&mut self, temp: f32) {
        self.settings.radio.temp = temp;
        self.settings.save();
    }
}

// Small functions
impl MusicController {
    /// Returns a track id of the first track in an album for a given album name
    /// The cover loading code works from track IDs so this works
    pub fn get_album_artwork(&self, album: String) -> usize {
        for (i, track) in self.all_tracks.iter().enumerate() {
            if strip_unnessecary(&track.album) == strip_unnessecary(&album) {
                return i;
            }
        }

        // As far as I know no one has millions of songs so this works
        return usize::MAX;
    }

    /// Returns the index of an album in the controller's inner list
    pub fn get_album_index(&self, album: &str) -> usize {
        self.albums.iter().position(|a| similar(album, a.0)).unwrap_or(0)
    }

    /// Toggles between playing and paused
    pub fn toggle_playing(&mut self) {
        send_music_msg(MusicMsg::Toggle);
        self.playing = !self.playing;
    }

    /// Unpauses the currently playing track
    pub fn play(&mut self) {
        send_music_msg(MusicMsg::Play);
        self.playing = true;
    }

    /// Pauses the currently playing track
    pub fn pause(&mut self) {
        send_music_msg(MusicMsg::Pause);
        self.playing = false;
    }

    /// Is the music player currently playing a track?
    pub fn playing(&self) -> bool {
        self.playing
    }

    /// Returns the index of the currently playing track
    pub fn current_track_idx(&self) -> usize {
        self.current_queue().current()
    }

    /// Gets a reference to the currently playing track
    pub fn current_track(&self) -> Option<&Track> {
        self.all_tracks.get(self.current_queue().current())
    }

    /// Gets a reference to a given track
    pub fn get_track(&self, idx: usize) -> Option<&Track> {
        self.all_tracks.get(idx)
    }

    /// Returns the current track's title
    pub fn current_track_title(&self) -> Option<&str> {
        Some(&self.current_track()?.title)
    }

    /// Returns the mood information of the currently playing track
    pub fn current_track_mood(&self) -> Option<Mood> {
        Some(self.current_track()?.mood.clone()?)
    }

    /// Returns the album of the currently playing track
    pub fn current_track_album(&self) -> Option<&str> {
        Some(&self.current_track()?.album)
    }

    /// Returns the artists of the currently playing track
    pub fn current_track_artist(&self) -> Option<&Vec<String>> {
        Some(&self.current_track()?.artists)
    }

    /// Returns the genres of the currently playing track
    pub fn current_track_genres(&self) -> Option<&Vec<String>> {
        Some(&self.current_track()?.genres)
    }

    /// Returns the index for the album of the currently playing track
    pub fn current_album_idx(&self) -> usize {
        let album = &self.current_track().unwrap().album;
        self.albums.iter().position(|e| *e.0 == *album).unwrap()
    }

    /// Tries to access the next track in the queue
    /// Returns None if the current track is the end of the queue
    pub fn next_up(&self) -> Option<Track> {
        let current_queue = &self.queues[self.current_queue];
        Some(
            self.all_tracks
                .get(*current_queue.cached_order.get(0)?)?
                .clone(),
        )
    }

    /// Gets a reference to a given queue
    pub fn get_queue(&self, idx: usize) -> &Queue {
        &self.queues[idx]
    }

    /// Gets a reference to the current queue
    pub fn current_queue(&self) -> &Queue {
        &self.queues[self.current_queue]
    }

    /// Gets a mutable reference to the current queue
    pub fn mut_current_queue(&mut self) -> &mut Queue {
        &mut self.queues[self.current_queue]
    }

    /// Sets audio player position
    pub fn set_pos(&self, pos: f64) {
        send_music_msg(MusicMsg::SetPos(pos));
    }
}

/// Applies setting weights to given features
pub fn genres_dist_from_vec(lhs: &TrackInfo, rhs: &TrackInfo, settings: &RadioSettings) -> f32 {
    let mfcc_sim = cosine_similarity(lhs.mfcc.clone(), rhs.mfcc.clone());
    let chroma_sim = cosine_similarity(lhs.mfcc.clone(), rhs.mfcc.clone());
    let spectral_sim = cosine_similarity(lhs.mfcc.clone(), rhs.mfcc.clone());
    let energy_sim = relative_similarity(lhs.energy, rhs.energy).min(1.0);
    let bpm_sim = relative_similarity(lhs.bpm, rhs.bpm).min(1.0);
    let zcr_sim = relative_similarity(lhs.zcr, rhs.zcr);

    (mfcc_sim * settings.mfcc_weight)
        + (chroma_sim * settings.chroma_weight)
        + (spectral_sim * settings.mfcc_weight)
        + (energy_sim * settings.energy_weight)
        + (bpm_sim * settings.bpm_weight)
        + (zcr_sim * settings.zcr_weight)
}

/// Implementation of cosine similarity
pub fn relative_similarity(lhs: f32, rhs: f32) -> f32 {
    1.0 - (((lhs + 0.01) / (rhs + 0.01)) / 2.0).abs()
}
