mod chroma;
mod mfcc;
pub mod utils;
pub use chroma::extract_chroma;
pub use mfcc::extract_mfcc;

use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use rodio::{Decoder, Source};
use crate::app::{track::{TrackInfo, Track}, embed::AutoEncoder};
use log::info;

pub fn generate_track_info(track: &Track, encoder: &AutoEncoder) -> TrackInfo {
    info!("generating analysis for track {track:?}");
    let genre_vec = encoder.genres_to_vec(track.genres.clone());
    let genre_space = encoder.encode(genre_vec);
    
    let started = Instant::now();
    let (samples, sample_rate) = load_samples(&track.file);
    // let duration_used = 10.0;
    // samples = samples[0..(sample_rate as f32 * duration_used) as usize].to_vec();
    info!("samples loaded in {:?}", started.elapsed());

    let started = Instant::now();
    let mfcc = extract_mfcc(&samples, sample_rate);
    info!("mfcc calculated in {:?}", started.elapsed());
    let started = Instant::now();
    let chroma = extract_mfcc(&samples, sample_rate);
    info!("chroma calculated in {:?}", started.elapsed());

    TrackInfo { genre_space, mfcc, chroma, bpm: 100, key: 0 }
}

pub fn load_samples(file_path: &str) -> (Vec<f32>, u32) {
    let file = File::open(file_path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let channels = source.channels();
    let sample_rate = source.sample_rate();
    info!("{sample_rate}");
    let started = Instant::now();
    // let samples: Vec<i16> = source.take(sample_rate as usize * 10).collect();
    // let samples: Vec<f32> = samples.into_iter().map(|n| n as f32).collect();
    let samples: Vec<f32> = source.convert_samples().collect();
    info!("samples calculated in {:?}", started.elapsed());

    info!("Total samples: {}", samples.len());

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

