mod chroma;
mod mfcc;
mod spectral;
mod tempo;
pub mod utils;
mod zcr;
pub use chroma::extract_chroma;
pub use mfcc::extract_mfcc;
pub use spectral::extract_spectral;
pub use tempo::extract_tempo;
pub use zcr::extract_zcr;

use crate::app::{
    embed::AutoEncoder,
    track::{Track, TrackInfo},
};
use log::info;
use ndarray::Array1;
use rodio::{Decoder, Source};
use std::io::BufReader;
use std::{fs::File, time::Duration};

pub fn generate_track_info(track: &Track, encoder: &AutoEncoder) -> TrackInfo {
    let genre_vec = encoder.genres_to_vec(track.genres.clone());
    let genre_space = encoder.encode(genre_vec);

    info!("{track:?}");
    let (samples, sample_rate) = load_samples(&track.file, Some((10.0, 10.0)));
    info!("{}", samples.len() as f32 / sample_rate as f32);

    let mfcc = extract_mfcc(&samples, sample_rate);
    let chroma = extract_mfcc(&samples, sample_rate);
    let spectral = extract_spectral(&samples, sample_rate);
    let energy = extract_energy(&samples).mean().unwrap_or_default();
    let bpm = extract_tempo(&samples, sample_rate);
    let zcr = extract_zcr(&samples, sample_rate);

    TrackInfo {
        genre_space,
        mfcc,
        chroma,
        spectral,
        bpm,
        energy,
        zcr,
        key: 0,
    }
}

pub fn load_samples(file_path: &str, duration: Option<(f64, f64)>) -> (Vec<f32>, u32) {
    if file_path.is_empty() {
        return (vec![0.0], 44000);
    }
    let file = File::open(file_path).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let channels = source.channels();
    let sample_rate = source.sample_rate();

    let samples: Vec<f32> = if let Some((duration, offset)) = duration {
        let sample_dur = source.total_duration().unwrap_or_default().as_secs_f64();
        let source = if sample_dur > duration + offset {
            source.skip_duration(Duration::from_secs_f64(offset))
        } else if sample_dur > duration {
            source.skip_duration(Duration::from_secs_f64(sample_dur - duration))
        } else {
            source.skip_duration(Duration::from_secs(0))
        };

        source
            .take((sample_rate as f64 * duration) as usize)
            .collect()
    } else {
        source.collect()
    };

    if samples.is_empty() {
        panic!("No samples were read from the file!");
    }

    let samples = if channels == 2 {
        samples
            .chunks_exact(2)
            .map(|s| (s[0] + s[1]) / 2.0)
            .collect()
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
