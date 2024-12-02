use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::fs::File;
use std::io::BufReader;

pub struct AudioPlayer {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        AudioPlayer { stream, stream_handle, sink }
    }

    pub fn play_track(&mut self, file_path: &str) {
        let file = BufReader::new(File::open(file_path).unwrap());
        let source = Decoder::new(file).unwrap();
        self.sink.skip_one();
        self.sink.append(source);
    }
}
