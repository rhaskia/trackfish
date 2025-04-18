mod chroma;
mod mfcc;
mod tempo;
mod spectral;
mod zcr;
pub mod utils;
pub use chroma::extract_chroma;
pub use mfcc::extract_mfcc;
use ndarray::Array1;
pub use spectral::extract_spectral;
pub use tempo::extract_tempo;
pub use zcr::extract_zcr;

use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use rodio::{Decoder, Source};
use crate::app::{track::{TrackInfo, Track}, embed::AutoEncoder};
use log::info;

pub fn generate_track_info(track: &Track, encoder: &AutoEncoder) -> TrackInfo {
    let genre_vec = encoder.genres_to_vec(track.genres.clone());
    let genre_space = encoder.encode(genre_vec);
    
    let (samples, sample_rate) = load_samples(&track.file, Some(10.0));

    let mfcc = extract_mfcc(&samples, sample_rate);
    let chroma = extract_mfcc(&samples, sample_rate);
    let spectral = extract_spectral(&samples, sample_rate);
    let energy = extract_energy(&samples).mean().unwrap_or(0.0);
    let bpm = extract_tempo(&samples, sample_rate);
    let zcr = extract_zcr(&samples, sample_rate);

    TrackInfo { genre_space, mfcc, chroma, spectral, bpm, energy, zcr, key: 0 }
}

pub fn load_samples(file_path: &str, duration: Option<f32>) -> (Vec<f32>, u32) {
    let file = File::open(file_path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let channels = source.channels();
    let sample_rate = source.sample_rate();
    let started = Instant::now();
    let samples: Vec<f32> = if let Some(duration) = duration {
        let samples: Vec<i16> = source.take((sample_rate as f32 * duration) as usize ).collect();
        samples.into_iter().map(|n| n as f32).collect()
    } else {
        source.convert_samples().collect()
    };

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

fn extract_energy(samples: &Vec<f32>) -> Array1<f32> {
    let frame_size = 2048;
    let hop_size = 2048;
    let mut rms_vals = Vec::new();
    let mut i = 0;

    while i + frame_size <= samples.len() {
        let frame = &samples[i..i + frame_size];
        let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
        rms_vals.push(rms);
        i += hop_size;
    }

    Array1::from_vec(rms_vals)
}
