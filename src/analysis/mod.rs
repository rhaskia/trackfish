mod chroma;
mod mfcc;
mod utils;

pub use chroma::extract_chroma;
pub use mfcc::extract_mfcc;

use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use rodio::{Decoder, Source};
use crate::app::{track::{TrackInfo, Track}, embed::AutoEncoder};
use ndarray::Array1;

pub fn generate_track_info(track: &Track, encoder: &AutoEncoder) -> TrackInfo {
    log::info!("generating analysis for track {track:?}");
    let genre_vec = encoder.genres_to_vec(track.genres.clone());
    let genre_space = encoder.encode(genre_vec);
    
    let started = Instant::now();
    let (mut samples, sample_rate) = load_samples(&track.file);
    let duration_used = 10.0;
    samples = samples[0..(sample_rate as f32 * duration_used) as usize].to_vec();
    println!("samples loaded in {:?}", started.elapsed());

    let started = Instant::now();
    let mfcc = extract_mfcc(&samples, sample_rate);
    println!("mfcc calculated in {:?}", started.elapsed());
    let started = Instant::now();
    let chroma = extract_mfcc(&samples, sample_rate);
    println!("chroma calculated in {:?}", started.elapsed());

    TrackInfo { genre_space: Array1::zeros(16), mfcc, chroma, bpm: 100, key: 0 }
}

pub fn load_samples(file_path: &str) -> (Vec<f32>, u32) {
    let file = File::open(file_path.clone()).unwrap();
    let mut source = Decoder::new(BufReader::new(file)).unwrap();

    let channels = source.channels();
    let sample_rate = source.sample_rate();
    println!("{sample_rate}");
    let samples: Vec<f32> = source.convert_samples().collect();
    let file = File::open(file_path).unwrap();
    let mut source = Decoder::new(BufReader::new(file)).unwrap();
    println!("{:?}, {:?}", source.next(), samples[0]);

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

