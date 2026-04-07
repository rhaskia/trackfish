use super::{
    playlist::get_playlist_files,
    playlist::Playlist,
    queue::{Listen, Queue, QueueType},
    settings::{RadioSettings, Settings, WeightMode},
    track::{Mood, Track, TrackInfo},
    utils::{similar, strip_unnessecary}, autoplaylist::AutoPlaylist,
};
use crate::database::{init_db, save_to_cache};
use crate::analysis::utils::cosine_similarity;
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
use dioxus::{prelude::*, stores::index::IndexWrite};

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
    UpdateInfo,
}

// Send message to AudioPlayer in thread
fn send_music_msg(msg: MusicMsg) {
    if let Some(tx) = MUSIC_PLAYER_ACTIONS.lock().unwrap().as_ref() {
        if let Err(e) = tx.send(msg) {
            info!("send error: {e:?}");
        }
    } else {
        info!("no MUSIC_PLAYER_ACTIONS set");
    }
}

type MappedQueue<Lens> = Store<Queue, MappedMutSignal<Queue, Lens, fn(&MusicController) -> &Queue, fn(&mut MusicController) -> &mut Queue>>;
type MappedTrack<Lens> = Store<Track, MappedMutSignal<Track, Lens, fn(&MusicController) -> &Track, fn(&mut MusicController) -> &mut Track>>;

#[derive(PartialEq, Clone, Store)]
pub struct MusicController {
    pub all_tracks: Vec<Track>,
    pub track_info: Vec<TrackInfo>,
    pub artists: HashMap<String, (String, usize)>,
    pub genres: HashMap<String, (String, usize)>,
    pub albums: HashMap<String, (usize, usize)>, // count, first track (for image purposes)
    pub listens: Vec<Listen>,
    pub shuffle: bool,
    pub playlists: Vec<Playlist>,
    pub autoplaylists: Vec<AutoPlaylist>,
    current_started: Instant,

    pub current_queue_index: usize,
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
            all_tracks: Vec::new(),
            track_info: Vec::new(),
            artists: HashMap::new(),
            genres: HashMap::new(),
            albums: HashMap::new(),
            listens: Vec::new(),
            current_started: Instant::now(),
            current_queue_index: 0,
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

        for i in 0..all_tracks.len() {
            for genre in all_tracks[i].genres.clone() {
                let stripped = strip_unnessecary(&genre);
                genres.entry(stripped).or_insert((genre, 0)).1 += 1;
            }

            for artist in all_tracks[i].artists.clone() {
                // some artists names seem to change captalization grr
                let stripped = strip_unnessecary(&artist);
                artists.entry(stripped).or_insert((artist, 0)).1 += 1;
            }

            albums.entry(all_tracks[i].album.clone()).or_insert((0, i)).0 += 1;
        }
        info!("Calculated weights in {:?}", started.elapsed());

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
            current_queue_index: 0,
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
            playing: false,
        };
        send_music_msg(MusicMsg::SetVolume(controller.settings.volume));

        info!("Loading playlists {:?}", started.elapsed());
        let files = get_playlist_files(&controller.settings.directory).unwrap();

        for file in files {
            let playlist = Playlist::load(&controller.settings.directory, &file, &controller.all_tracks);
            controller.playlists.push(playlist);
        }
        info!("Loaded playlists in {:?}", started.elapsed());
        let files = get_playlist_files(&controller.settings.directory).unwrap();

        for file in files {
            let playlist = Playlist::load(&controller.settings.directory, &file, &controller.all_tracks);
            controller.playlists.push(playlist);
        }

        info!("Loaded playlists in {:?}", started.elapsed());

        if let Some(track) = controller.all_tracks.get(controller.queues[controller.current_queue_index].current()) {
            send_music_msg(MusicMsg::PlayTrack(track.file.clone()));
            info!("Started track {track:?} in {:?}", started.elapsed());
        }

        controller
    }
}

#[store(pub)]
impl<Lens> Store<MusicController, Lens> {
    fn delete_playlist(&mut self, playlist: usize) {
        let path = self.playlists().get(playlist).unwrap().read().file.clone();
        std::fs::remove_file(path).unwrap();
        self.playlists().write().remove(playlist);
    }

    /// Loads all playlists in the music directory (.m3u files)
    fn load_playlists(&mut self) {
        let files = get_playlist_files(&self.settings().read().directory).unwrap();

        for file in files {
            let playlist = Playlist::load(&self.settings().read().directory, &file, &self.all_tracks().read());
            self.playlists().write().push(playlist);
        }
    }

    /// Loads all autoplaylists saved in cache (.auto files)
    fn load_autoplaylists(&mut self) {
        for entry in std::fs::read_dir(Settings::dir()).unwrap() {
            let path = entry.unwrap().path();

            if path.is_file() {
                if path.extension().unwrap_or_default().to_str().unwrap_or_default() == "auto" {
                    match AutoPlaylist::load(path) {
                        Ok(ap) => self.autoplaylists().write().push(ap),
                        Err(e) => error!("{e:?}"),
                    }
                }
            }
        }
    }

    /// Deletes an autoplaylist from storage and memory
    fn rename_autoplaylist(&mut self, autoplaylist: usize, name: String) {
        let path = self.autoplaylists().get(autoplaylist).unwrap().read().dir();
        std::fs::remove_file(path).unwrap();
        self.autoplaylists().get(autoplaylist).unwrap().write().name = name;
        self.autoplaylists().get(autoplaylist).unwrap().write().save();
    }

    /// Deletes an autoplaylist from storage and memory
    fn delete_autoplaylist(&mut self, autoplaylist: usize) {
        let path = self.autoplaylists().get(autoplaylist).unwrap().read().dir();
        std::fs::remove_file(path).unwrap();
        self.autoplaylists().write().remove(autoplaylist);
    }

    /// Saves a playlist in the M3U format
    fn save_playlist(&mut self, playlist: usize) {
        let playlist = self.playlists().read()[playlist].clone();
        let relative_paths: Vec<String> = playlist
            .tracks
            .iter()
            .map(|t| relative_path(&self.all_tracks().read()[*t].file, &self.settings().read().directory))
            .collect();

        let file = String::from("#EXTM3U\n#PLAYLIST:")
            + &playlist.name
            + "\n"
            + &relative_paths.join("\n\n");
        std::fs::write(&playlist.file, file).unwrap();
    }

    /// Renames all instances of a genre on tracks
    fn rename_genre(&mut self, old_genre: String, new_genre: String) {
        info!("Renaming genre {old_genre} to {new_genre}");
        for i in 0..self.all_tracks().read().len() {
            let track = self.all_tracks().get(i).unwrap();
            info!("{i}");
            
            if track.read().genres.contains(&old_genre) {
                info!("Updating track {i}'s {old_genre} to {new_genre}");
                let mut new_tag = track();

                if let Some(j) = new_tag.genres.iter().position(|g| g == &old_genre) {
                    assert_eq!(new_tag.genres[j], old_genre);
                    new_tag.genres[j] = new_genre.to_string();
                    let conn = init_db().unwrap();
                    self.update_tag(&conn, i, new_tag);
                }
            }
        }
    }

    /// Plays a given track
    fn play_track(&mut self, idx: usize) {
        if let Some(current_track) = self.current_track() {
            let listen = Listen::new(
                self.current_track_idx(),
                self.current_started()(),
                current_track.read().len,
                self.progress_secs()()
            );
            
            self.listens().write().push(listen);
        }

        *self.current_started().write() = Instant::now();
        *self.progress_secs().write() = 0.0;

        send_music_msg(MusicMsg::PlayTrack(self.all_tracks().read()[idx].file.clone()));
        info!("sent music msg to thread");
    }

    /// Returns the current track weights, or average track weights accross the queue
    fn get_space(&mut self) -> TrackInfo {
        match self.settings().read().radio.weight_mode {
            WeightMode::First => self.track_info().read()[self.current_queue().read().cached_order[0]].clone(),
            WeightMode::Last => {
                self.track_info().read()[*self.current_queue().read().cached_order.iter().last().unwrap()].clone()
            }
            WeightMode::Average => {
                let mut tracks = Vec::new();

                // Introduce count later?
                let count = self.current_queue().read().cached_order.len();
                for i in (count.max(10) - 10)..count {
                    tracks.push(
                        self.track_info().read()
                            .get(self.current_queue().read().cached_order[i])
                            .cloned()
                            .unwrap_or_default(),
                    );
                }

                TrackInfo::average(tracks)
            }
        }
    }

    /// Returns all given weights for tracks in the player
    fn get_weights(&mut self) -> Array1<f32> {
        let space = self.get_space();

        let mut weights = Array1::from_vec(vec![0.0; self.all_tracks().read().len()]);
        let mut dists: Vec<(usize, f32)> = self
            .track_info()
            .read()
            .iter()
            .map(|track| genres_dist_from_vec(&track, &space, &self.settings().read().radio))
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
            if self.current_queue().read().cached_order.contains(song) {
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

        for i in 0..self.all_tracks().read().len() {
            let current_idx = self.current_queue().read().current();
            if similar(
                &self.all_tracks().read()[current_idx].album,
                &self.all_tracks().read()[i].album,
            ) {
                weights *= self.settings().read().radio.album_penalty;
            }

            if self.all_tracks().get(current_idx).unwrap().read().shared_artists(&*self.all_tracks().get(i).unwrap().read()) > 0 {
                weights *= self.settings().read().radio.artist_penalty;
            }
        }

        // TODO: weights for each feature used in weighting

        weights
    }

    /// Returns the next 'similar' track to play
    fn next_similar(&mut self) -> usize {
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
    fn skipback(&mut self) {
        if self.progress_secs()() < 5.0 {
            if self.queues().read()[self.current_queue_index()()].current_track == 0 {
                return;
            }
            let last = self.queues().read()[self.current_queue_index()()].current_track - 1;
            self.queues().get(self.current_queue_index()()).unwrap().write().current_track = last;
        }

        self.play_track(self.queues().read()[self.current_queue_index()()].current());
    }

    /// Skips the current track in the queue, or skips to the next queue if at end of queue
    fn skip(&mut self) {
        if self.all_tracks().read().is_empty() {
            log::info!("No track to skip to");
            return;
        }

        // next track exists in queue
        if let Some(next) = self.current_queue().read()
            .cached_order
            .get(self.current_queue().read().current_track + 1)
            .cloned()
        {
            self.current_queue().write().current_track += 1;
            self.play_track(next);
            return;
        }

        match self.current_queue()().queue_type {
            QueueType::Radio(_) => {
                let next = self.next_similar();
                let cqi = self.current_queue_index()();
                self.queues().get(cqi).unwrap().write().current_track += 1;
                self.queues().get(cqi).unwrap().write().cached_order.push(next);
                info!("hi3");
                self.play_track(next);
                info!("hi2");
            }
            _ => {
                if self.queues().read().len() > self.current_queue_index()() + 1 {
                    *self.current_queue_index().write() += 1;
                    // TODO: shuffle next queue if needed
                    self.play_track(self.current_queue().read().track(0))
                }
            }
        }

        info!("successfully skipped song");
    }

    /// Sets the current queue playing and at which track
    fn set_queue_and_track(&mut self, queue: usize, track: usize) {
        self.current_queue_index().set(queue);
        self.queues().get(queue).unwrap().write().current_track = track;
        self.play_track(self.queues().get(queue).unwrap().read().cached_order[track]);
    }

    /// Returns tracks matching a certain QueueType
    fn get_matching(&self, queue_type: QueueType) -> Vec<usize> {
        if queue_type == QueueType::AllTracks {
            return (0..self.all_tracks().read().len()).collect();
        }

        self.all_tracks().read()
            .iter()
            .enumerate()
            .filter(|(_, track)| track.matches(queue_type.clone()))
            .map(|(index, _)| index)
            .collect()
    }

    /// Removes a queue from the queue list and moves to another queue
    /// TODO: some better way of choosing the queue to shift to 
    fn remove_queue(&mut self, queue: usize) {
        if self.current_queue_index()() == queue && queue != 0 {
            *self.current_queue_index().write() -= 1;
        }
        self.queues().write().remove(queue);
    }

    /// Creates a playlist using tracks in a given queue
    fn queue_to_playlist(&mut self, queue: usize) {
        let queue = self.queues().read()[queue].clone();
        let mut playlist = Playlist::new(
            format!("{}", queue.queue_type),
            self.settings().read().directory.clone(),
        );
        playlist.tracks = queue.cached_order;
        self.playlists().write().push(playlist);
        self.save_playlist(self.playlists().read().len() - 1);

        // TODO replace queue with playlist queue?
    }

    /// Adds a list of tracks to a given queue
    fn add_tracks_to_queue(&mut self, queue: usize, tracks: Vec<usize>) {
        self.queues().get(queue).unwrap().write().cached_order.extend(tracks);
    }

    /// Adds a list of tracks to a given playlist
    fn add_tracks_to_playlist(&mut self, playlist: usize, tracks: Vec<usize>) {
        self.playlists().get(playlist).unwrap().write().tracks.extend(tracks);
    }

    /// Deletes a track and updates controller information about album/artist/genre amounts 
    fn delete_track(&mut self, conn: &Connection, track: usize) {
        std::fs::remove_file(self.all_tracks().read()[track].file.clone()).unwrap();
        let album = self.all_tracks().read()[track].album.clone();
        let artists = self.all_tracks().read()[track].artists.clone();
        let genres = self.all_tracks().read()[track].genres.clone();

        if self.albums().get(album.clone()).unwrap().read().0 == 1 {
            self.albums().write().remove(&album);
        } else {
            if let Some(mut val) = self.albums().get(album.clone()) { val.write().0 -= 1; };
        }

        for artist in artists {
            let stripped = strip_unnessecary(&artist);
            if self.artists().get(stripped.clone()).unwrap().read().1 == 1 {
                self.artists().write().remove(&stripped);
            } else {
                if let Some(mut val) = self.artists().get(stripped) { val.write().1 -= 1; };
            }
        }

        for genre in genres {
            let stripped = strip_unnessecary(&genre);
            if self.genres().get(stripped.clone()).unwrap().read().1 == 1 {
                self.genres().write().remove(&genre);
            } else {
                if let Some(mut val) = self.genres().get(genre) { val.write().1 -= 1; };
            }
        }

        crate::database::remove_track_from_database(&conn, &self.all_tracks().read()[track].file).unwrap();

        info!("successfully deleted track {:?}", self.all_tracks().read()[track].title);

        // TODO: some better way of removing tracks during runtime
        *self.all_tracks().get(track).unwrap().write() = Track::default();
    }   

    /// Updates track tag in memory and saves it to storage 
    fn update_tag(&mut self, conn: &Connection, track: usize, tag: Track) {
        info!("db2");
        if tag == self.all_tracks().read()[track] {
            info!("Nothing to update with tag");
            return;
        }

        let old_track = self.all_tracks().get(track).unwrap(); 
        let old_album = old_track.read().album.clone();
        let old_artists = old_track.read().artists.clone();
        let old_genres = old_track.read().genres.clone();
        info!("db2");

        if old_album != tag.album {
            if self.albums().read()[&old_album].0 == 1 {
                self.albums().write().remove(&old_album);
            } else {
                if let Some(mut val) = self.albums().get(old_album.clone()) { val.write().0 -= 1; };
            }

            if self.albums().read().contains_key(&tag.album) {
                if let Some(mut val) = self.albums().get(tag.album.clone()) { val.write().0 += 1; };
            } else {
                self.albums().write().insert(tag.album.clone(), (1, track));
            }
        }

        if old_artists != tag.artists {
            for artist in old_artists {
                let stripped = strip_unnessecary(&artist);
                if self.artists().read()[&stripped].1 == 1 {
                    self.artists().write().remove(&stripped);
                } else {
                    if let Some(mut val) = self.artists().get(stripped) { val.write().1 -= 1; };
                }
            }

            for artist in &tag.artists {
                let stripped = strip_unnessecary(&artist);
                if self.artists().read().contains_key(&stripped) {
                    if let Some(mut val) = self.artists().get(stripped) { val.write().1 += 1; };
                } else {
                    self.artists().write().insert(stripped, (artist.to_string(), 1));
                }
            }
        }

        if old_genres != tag.genres {
            for genre in old_genres {
                let stripped = strip_unnessecary(&genre);
                if self.genres().read()[&stripped].1 == 1 {
                    self.genres().write().remove(&stripped);
                } else {
                    if let Some(mut val) = self.genres().get(stripped) { val.write().1 -= 1; };
                }
            }

            for genre in &tag.genres {
                let stripped = strip_unnessecary(&genre);
                if self.genres().read().contains_key(&stripped) {
                    if let Some(mut val) = self.genres().get(stripped) { val.write().1 += 1; };
                } else {
                    self.genres().write().insert(stripped, (genre.to_string(), 1));
                }
            }
        }
        info!("updated");

        tag.save_to_disk(&conn).unwrap();

        *self.all_tracks().get(track).unwrap().write() = tag;
    }

    /// Starts an artist queue at no specific starting track
    fn add_artist_queue(&mut self, artist: String) {
        let tracks = self.get_tracks_where(|track| track.artists.contains(&artist));
        self.queues().write()
            .push(Queue::new(QueueType::Artist(artist), tracks));
        *self.current_queue_index().write() = self.queues().read().len() - 1;
    }
    
    /// Starts an album queue starting with a specified track
    fn play_album_at(&mut self, album: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.album == album);
        self.add_queue_at(tracks, QueueType::Album(album.clone()), track);
    }

    /// Starts an genre queue starting with a specified track
    fn play_genre_at(&mut self, genre: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.has_genre(&genre));
        self.add_queue_at(tracks, QueueType::Genre(genre.clone()), track);
    }

    /// Starts an artist queue starting with a specified track
    fn play_artist_at(&mut self, artist: String, track: usize) {
        let tracks = self.get_tracks_where(|track| track.has_artist(&artist));
        self.add_queue_at(tracks, QueueType::Artist(artist.clone()), track);
    }

    /// Starts a radio queue with a specified starting track
    fn start_radio(&mut self, track: usize) {
        let track_name = self.all_tracks().read()[track].title.clone();
        self.add_queue_at(vec![track], QueueType::Radio(track_name), track);
    }

    /// Starts a playlist, with a given track to start
    fn play_playlist_at(&mut self, playlist: usize, track: usize) {
        self.add_queue_at(
            self.playlists().read()[playlist].tracks.clone(),
            QueueType::Playlist(self.playlists().read()[playlist].name.clone(), playlist),
            track,
        );
    }

    /// Starts an autoplaylist, with a given track to start
    fn play_autoplaylist_at(&mut self, tracks: Vec<usize>, autoplaylist: usize, track: usize) {
        self.add_queue_at(
            tracks,
            QueueType::AutoPlaylist(self.autoplaylists().read()[autoplaylist].name.clone(), autoplaylist),
            track,
        );
    }

    /// Starts a given queue with some tracks at a specific track
    fn add_queue_at(&mut self, tracks: Vec<usize>, queue: QueueType, track: usize) {
        let mut tracks = tracks;
        if self.shuffle()() {
            tracks = shuffle_with_first(tracks, track);
        }

        info!("{track}");
        let track_idx = tracks.iter().position(|e| *e == track).unwrap();

        for i in 0..self.queues().read().len() {
            if self.queues().read()[i].queue_type == queue {
                self.queues().get(i).unwrap().write().cached_order = tracks;
                self.queues().get(i).unwrap().write().current_track = track_idx;
                self.current_queue_index().set(i);
                self.play_track(track);
                self.play();
                info!("{}, {}", self.queues().read()[i].current(), track);
                return;
            }
        }

        self.queues().write().push(Queue::new(queue, tracks));
        self.current_queue_index().set(self.queues().read().len() - 1);
        self.queues().get(self.current_queue_index()()).unwrap().write().current_track = track_idx;
        self.play_track(track);
        self.play();
    }

    /// Add a queue containing all tracks, with a given track to start
    fn add_all_queue(&mut self, track: usize) {
        let tracks = (0..self.all_tracks().read().len()).collect();
        self.add_queue_at(tracks, QueueType::AllTracks, track);
    }

    /// Get tracks that fit a given conditional, using a supplied Fn
    fn get_tracks_where<F>(&self, condition: F) -> Vec<usize>
    where
        F: Fn(&Track) -> bool,
    {
        self.all_tracks().read()
            .iter()
            .enumerate()
            .filter(|(_, track)| condition(*track))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Toggles between shuffled and unshuffled in all queues
    fn toggle_shuffle(&mut self) {
        if self.shuffle()() {
            // unshuffle queues
            for queue in &mut *self.queues().write() {
                let current = queue.cached_order[queue.current_track];

                match queue.queue_type {
                    QueueType::Radio(_) => {}
                    QueueType::Album(_) => queue.cached_order.sort_by(|a, b| {
                        self.all_tracks().read()[*a]
                            .trackno
                            .cmp(&self.all_tracks().read()[*b].trackno)
                    }),
                    _ => queue.cached_order.sort_by(|a, b| a.cmp(b)),
                }

                // Keep same track playing
                let new_idx = queue.cached_order.iter().position(|n| *n == current);
                queue.current_track = new_idx.unwrap_or(0);
            }
        } else {
            for queue in &mut *self.queues().write() {
                if let QueueType::Radio(_) = queue.queue_type {
                    // Painful to try and unshuffle radio queues
                    continue;
                }

                queue.cached_order =
                    shuffle_with_first(queue.cached_order.clone(), queue.current());
                queue.current_track = 0;
            }
        }

        self.shuffle().toggle();
    }

    /// Adds a track to the spot after the current track in queue
    fn play_next(&mut self, track: usize) {
        let position = self.current_queue().read().current_track;
        self.current_queue().write()
            .cached_order
            .insert(position + 1, track);
    }

    /// Adds a track to a given playlist
    fn add_to_playlist(&mut self, playlist: usize, track: usize) {
        let file = relative_path(&self.all_tracks().read()[track].file, &self.settings().read().directory);
        info!("{file}");
        self.playlists().get(playlist).unwrap().write().tracks.push(track);
        self.playlists().get(playlist).unwrap().write().track_paths.push(file);
        self.save_playlist(playlist);
    }

    /// Find likely duplicate tracks
    fn find_duplicates(&self) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut titles: HashMap<String, usize> = HashMap::new();

        for i in 0..self.all_tracks().read().len() {
            *titles.entry(strip_unnessecary(&self.all_tracks().read()[i].title)).or_default() += 1;
        }

        for (key, value) in titles {
            if value <= 1 { 
                continue;
            }

            let mut similars = Vec::new();

            for i in 0..self.all_tracks().read().len() {
                if key == strip_unnessecary(&self.all_tracks().read()[i].title) {
                    similars.push(i)
                }
            }
            
            results.push(similars);
        }

        results
    }

    /// Sets the volume of the music player and saves it to storage
    fn set_volume(&mut self, volume: f32) {
        self.settings().write().volume = volume;
        send_music_msg(MusicMsg::SetVolume(volume));
        self.settings().read().save();
        info!("Set volume to {volume}");
    }

    /// Sets the music directory, and saves it to storage
    fn set_directory(&mut self, new_dir: String) {
        self.settings().write().directory = new_dir;
        self.settings().read().save();
        // Manage loading new tracks
    }

    /// Sets the 'temperature' of the reccomendation system
    fn set_temp(&mut self, temp: f32) {
        self.settings().write().radio.temp = temp;
        self.settings().read().save();
    }

    /// Returns the index of an album in the controller's inner list
    fn get_album_index(&self, album: &str) -> usize {
        self.albums().read().iter().position(|a| similar(album, a.0)).unwrap_or(0)
    }

    /// Toggles between playing and paused
    fn toggle_playing(&mut self) {
        send_music_msg(MusicMsg::Toggle);
        self.playing().toggle();
    }

    /// Unpauses the currently playing track
    fn play(&mut self) {
        send_music_msg(MusicMsg::Play);
        self.playing().set(true);
    }

    /// Pauses the currently playing track
    fn pause(&mut self) {
        send_music_msg(MusicMsg::Pause);
        self.playing().set(true);
    }

    /// Is the music player currently playing a track?
    fn is_playing(&self) -> bool {
        self.playing()()
    }

    /// Returns the index of the currently playing track
    fn current_track_idx(&self) -> usize {
        self.queues().get(self.current_queue_index()()).unwrap().read().current()
    }

    /// Gets a reference to the currently playing track
    fn current_track(&self) -> Option<Store<Track, IndexWrite<usize, MappedMutSignal<std::vec::Vec<Track>, Lens, for<'a> fn(&'a MusicController) -> &'a Vec<Track>, for<'a> fn(&'a mut MusicController) -> &'a mut std::vec::Vec<Track>>>>> {
        self.all_tracks().get(self.current_track_idx())
    }

    /// Gets a reference to a given track
    fn get_track(&self, idx: usize) -> Option<Store<Track, IndexWrite<usize, MappedMutSignal<std::vec::Vec<Track>, Lens, for<'a> fn(&'a MusicController) -> &'a Vec<Track>, for<'a> fn(&'a mut MusicController) -> &'a mut std::vec::Vec<Track>>>>> {
        self.all_tracks().get(idx)
    }

    /// Returns the current track's title
    fn current_track_title(&self) -> Option<String> {
        Some(self.current_track()?.read().title.clone())
    }

    /// Returns the mood information of the currently playing track
    fn current_track_mood(&self) -> Option<Mood> {
        Some(self.current_track()?.read().mood.clone()?)
    }

    /// Returns the album of the currently playing track
    fn current_track_album(&self) -> Option<String> {
        Some(self.current_track()?.read().album.clone())
    }

    /// Returns the artists of the currently playing track
    fn current_track_artist(&self) -> Option<Vec<String>> {
        Some(self.current_track()?.read().artists.clone())
    }

    /// Returns the genres of the currently playing track
    fn current_track_genres(&self) -> Option<Vec<String>> {
        Some(self.current_track()?.read().genres.clone())
    }

    /// Returns the index for the album of the currently playing track
    fn current_album_idx(&self) -> usize {
        let album = self.current_track().unwrap().read().album.clone();
        self.albums().read().iter().position(|e| *e.0 == album).unwrap()
    }

    /// Gets a reference to a given queue
    fn get_queue(&self, idx: usize) -> Store<Queue, IndexWrite<usize, MappedMutSignal<Vec<Queue>, Lens, for<'a> fn(&'a MusicController) -> &'a Vec<Queue>, for<'a> fn(&'a mut MusicController) -> &'a mut Vec<Queue>>>> {
        self.queues().get(idx).unwrap()
    }

    /// Gets a reference to the current queue
    fn current_queue(&self) -> Store<Queue, IndexWrite<usize, MappedMutSignal<Vec<Queue>, Lens, for<'a> fn(&'a MusicController) -> &'a Vec<Queue>, for<'a> fn(&'a mut MusicController) -> &'a mut Vec<Queue>>>> {
        self.queues().get(self.current_queue_index()()).unwrap()
    }

    /// Tries to access the next track in the queue
    /// Returns None if the current track is the end of the queue
    fn next_up(&self) -> Option<Track> {
        Some(
            self.all_tracks().read()
                .get(*self.queues().get(self.current_queue_index()()).unwrap().read().cached_order.get(0)?)?
                .clone(),
        )
    }

    /// Sets audio player position
    fn set_pos(&mut self, pos: f64) {
        self.progress_secs().set(pos);
        send_music_msg(MusicMsg::SetPos(pos));
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
