use log::info;
use rodio::queue::SourcesQueueOutput;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub struct AudioPlayer {
    sink: Sink,
    _sources: SourcesQueueOutput,
    current_song_len: f64,
}

impl PartialEq for AudioPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.current_song_len == other.current_song_len
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (sink, _sources) = Sink::new();

        AudioPlayer {
            sink,
            _sources,
            current_song_len: 1.0,
        }
    }

    pub fn play_track(&mut self, file_path: &str) {
        info!("Playing track: {file_path:?}");
        let file = BufReader::new(File::open(file_path).unwrap());
        let source = Decoder::new(file).unwrap();
        self.current_song_len = source
            .total_duration()
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();

        let was_paused = self.sink.is_paused();
        self.sink.clear();
        self.sink.append(source);
        if !was_paused {
            self.sink.play();
        }
        info!("{}", self.sink.empty());
        info!("Track successfully played");
    }

    pub fn toggle_playing(&mut self) {
        info!("{}", self.sink.is_paused());
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
        let _ = self.sink.try_seek(Duration::from_secs_f64(pos));
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume)
    }
}
