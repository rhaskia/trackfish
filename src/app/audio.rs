use log::info;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub struct AudioPlayer {
    sink: Sink,
    _stream_handle: OutputStream,
    current_song_len: f64,
}

impl PartialEq for AudioPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.current_song_len == other.current_song_len
    }
}

impl AudioPlayer {
    /// New audio player using the default device stream
    pub fn new() -> Self {
        let _stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(&_stream_handle.mixer());

        AudioPlayer {
            sink,
            _stream_handle,
            current_song_len: 1.0,
        }
    }

    /// Plays a new track from file into the audio sink
    pub fn play_track(&mut self, file_path: &str) -> f64 {
        info!("Playing track: {file_path:?}");
        let f = File::open(file_path).unwrap();
        let _len = f.metadata().unwrap().len();
        let file = BufReader::new(f);

        let source = Decoder::builder()
            .with_data(file)
            .with_seekable(true)
            .build()
            .unwrap();

        self.current_song_len = source
            .total_duration()
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();

        let was_paused = self.sink.is_paused();
        self.sink.clear();
        self.sink.append(source);
        self.sink.play();

        if !was_paused {
            self.sink.play();
        }

        // Seek to 0 to make sure the track starts from the beginning
        self.set_pos(0.0);
        info!("Track successfully played");
        self.current_song_len
    }

    /// Toggles the audio device from playing the current track
    pub fn toggle_playing(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn play(&mut self) {
        self.sink.play();
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn progress_percent(&self) -> f64 {
        self.sink.get_pos().as_secs_f64() / self.current_song_len
    }

    pub fn progress_secs(&self) -> f64 {
        self.sink.get_pos().as_secs_f64()
    }

    pub fn track_ended(&self) -> bool {
        self.sink.empty()
    }

    pub fn song_length(&self) -> f64 {
        self.current_song_len
    }

    pub fn set_pos(&mut self, pos: f64) {
        let try_seek = self.sink.try_seek(Duration::from_secs_f64(pos));
        if let Err(seek_error) = try_seek {
            info!("Recieved seek error: {seek_error:?}");
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume)
    }
}
