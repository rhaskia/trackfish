use log::info;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub struct AudioPlayer {
    _stream: Option<(OutputStream, OutputStreamHandle)>,
    sink: Option<Sink>,
    current_song_len: f64,
}

impl PartialEq for AudioPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.current_song_len == other.current_song_len
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        let _stream = OutputStream::try_default();
        let sink = if let Ok(ref _stream) = _stream {
            Sink::try_new(&_stream.1).ok()
        } else {
            None
        };

        AudioPlayer {
            _stream: _stream.ok(),
            sink,
            current_song_len: 1.0,
        }
    }

    pub fn try_new_sink(&mut self) -> Option<&Sink> {
        if self.sink.is_some() {
            return self.sink.as_ref();
        }

        let _stream = OutputStream::try_default();
        let sink = if let Ok(ref _stream) = _stream {
            Sink::try_new(&_stream.1).ok()
        } else {
            None
        };

        self._stream = _stream.ok();
        self.sink = sink;

        self.sink.as_ref()
    }

    pub fn play_track(&mut self, file_path: &str) {
        info!("Playing track: {file_path:?}");
        let file = BufReader::new(File::open(file_path).unwrap());
        let source = Decoder::new(file).unwrap();
        self.current_song_len = source
            .total_duration()
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();

        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        let was_paused = sink.is_paused();
        sink.clear();
        sink.append(source);
        if !was_paused {
            sink.play();
        }
        info!("{}", sink.empty());
        info!("Track successfully played");
    }

    pub fn toggle_playing(&mut self) {
        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        if sink.is_paused() {
            sink.play();
        } else {
            sink.pause();
        }
    }

    pub fn play(&mut self) {
        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        sink.play();
    }

    pub fn pause(&mut self) {
        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        sink.pause();
    }

    pub fn playing(&self) -> bool {
        let sink = if let Some(sink) = &self.sink {
            sink
        } else {
            return false;
        };

        !sink.is_paused()
    }

    pub fn progress_percent(&self) -> f64 {
        let sink = if let Some(sink) = &self.sink {
            sink
        } else {
            return 0.0;
        };

        sink.get_pos().as_secs_f64() / self.current_song_len
    }

    pub fn progress_secs(&self) -> f64 {
        let sink = if let Some(sink) = &self.sink {
            sink
        } else {
            return 0.0;
        };

        sink.get_pos().as_secs_f64()
    }

    pub fn track_ended(&self) -> bool {
        let sink = if let Some(sink) = &self.sink {
            sink
        } else {
            return false;
        };

        sink.empty()
    }

    pub fn song_length(&self) -> f64 {
        self.current_song_len
    }

    pub fn set_pos(&mut self, pos: f64) {
        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        let _ = sink.try_seek(Duration::from_secs_f64(pos));
    }

    pub fn set_volume(&mut self, volume: f32) {
        let sink = if let Some(sink) = self.try_new_sink() {
            sink
        } else {
            return;
        };

        sink.set_volume(volume)
    }
}
