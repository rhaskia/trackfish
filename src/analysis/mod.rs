mod chroma;
mod mfcc;

pub use chroma::extract_chroma;
pub use mfcc::extract_mfcc;

use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, Source};

pub fn load_samples(file: &str) -> (Vec<f32>, u32) {
    let file = File::open(file).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let channels = source.channels();
    let sample_rate = source.sample_rate();
    println!("{sample_rate}");
    let samples: Vec<f32> = source.convert_samples().collect();

    println!("Total samples: {}", samples.len());

    if samples.is_empty() {
        panic!("No samples were read from the file!");
    }

    let samples = if channels == 2 {
        samples.chunks_exact(2).map(|s| (s[0] + s[1]) / 2.0).collect()
    } else {
        samples.clone()
    };

    (samples, sample_rate)
}

pub fn linear_resample(audio_data: &Vec<f32>, input_rate: usize, output_rate: usize) -> Vec<f32> {
    let ratio = output_rate as f32 / input_rate as f32;
    let new_len = (audio_data.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let src_index = i as f32 / ratio;
        let idx = src_index.floor() as usize;
        let frac = src_index - idx as f32;
        
        let v0 = audio_data.get(idx).cloned().unwrap_or(0.0);
        let v1 = audio_data.get(idx + 1).cloned().unwrap_or(v0);
        
        resampled.push(v0 + frac * (v1 - v0));
    }
    resampled
}

