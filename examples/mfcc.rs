use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use aubio::MFCC;
use aubio::FFT;

fn std_dev(arr: Vec<f32>, mean: f32) -> f32 {
    let sum_of_squares = arr.iter().map(|x| (x - mean).powi(2)).sum::<f32>();
    (sum_of_squares / arr.len() as f32).sqrt()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("E:\\Music\\Lemon [H5ZPCcnLXt4].mp3")?;
    let source = Decoder::new(BufReader::new(file))?;

    let channels = source.channels();
    let sample_rate = source.sample_rate();
    let samples: Vec<f32> = source.convert_samples().collect();

    println!("Total samples: {}", samples.len());

    if samples.is_empty() {
        panic!("No samples were read from the file!");
    }

    let mut mono_samples: Vec<f32> = if channels == 2 {
        samples.chunks_exact(2).map(|s| (s[0] + s[1]) / 2.0).collect()
    } else {
        samples.clone()
    };

    let frame_size = 1024;
    let mfcc_len = 13;

    let result = calculate_mfcc(&mut mono_samples, sample_rate).unwrap();
    let mean = result.iter().sum::<f32>() / mfcc_len as f32;
    let std = std_dev(result.clone(), mean);
    let result = result.iter().map(|n| (n - mean) / std).collect::<Vec<f32>>();
    println!("{result:?}");

    let result = extract_chroma(&mono_samples, sample_rate as usize);
    println!("{result:?}");

    Ok(())
}

use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

fn extract_chroma(audio_data: &[f32], sample_rate: usize) -> Vec<f32> {
    let frame_size = 2048;
    let num_coefficients = 12;
    let hop_size = 512;
    let mut chroma_vectors = Vec::new();

    let mut fft_planner = FftPlanner::new();
    let fft = fft_planner.plan_fft_forward(frame_size);

    for frame in audio_data.chunks(hop_size) {
        let mut buffer: Vec<Complex<f32>> = frame.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
        buffer.resize(frame_size, Complex { re: 0.0, im: 0.0 });

        fft.process(&mut buffer);

        let mut chroma = [0.0; 12];
        for (k, bin) in buffer.iter().enumerate() {
            let frequency = k as f32 * sample_rate as f32 / frame_size as f32;
            if frequency > 0.0 {
                let pitch_class = (12.0 * (frequency / 440.0).log2()).rem_euclid(12.0);
                let index = pitch_class.round() as usize % 12;
                chroma[index] += bin.norm();
            }
        }

        let norm: f32 = chroma.iter().sum();
        if norm > 0.0 {
            for value in &mut chroma {
                *value /= norm;
            }
        }

        chroma_vectors.push(chroma);
    }
    
    let mut mean_chroma: Vec<f32> = vec![0.0; num_coefficients];

    for i in 0..num_coefficients {
        for j in 0..chroma_vectors.len() {
            mean_chroma[i] += chroma_vectors[j][i];
        }

        mean_chroma[i] /= chroma_vectors.len() as f32;
    }

    mean_chroma
}

fn calculate_mfcc(buffer: &mut Vec<f32>, sample_rate: u32) -> Vec<f32> {
    let fft_size = 1024;
    let num_coefficients = 13;
    let num_filters = 20;

    let num_blocks = (buffer.len() as f32 / fft_size as f32).floor() as usize;
    if num_blocks == 0 {
        buffer.resize(fft_size, 0.0);
    }

    let mut fft = FFT::new(fft_size).map_err(|e| e.to_string()).unwrap();
    let mut fft_scratch: Vec<f32> = vec![0.0; fft_size];

    let mut mfcc = MFCC::new(fft_size, num_filters, num_coefficients, sample_rate).unwrap();
    let mut mean_mfcc: Vec<f32> = vec![0.0; num_coefficients];
    let mut mfcc_scratch: Vec<f32> = vec![0.0; num_coefficients];

    for block_index in 0..num_blocks {
        let start = block_index * fft_size;
        let buf = &buffer[start..];

        fft.do_(buf, &mut fft_scratch).unwrap();
        mfcc.do_(&fft_scratch, &mut mfcc_scratch).unwrap();

        for (new, mean) in mfcc_scratch.iter().zip(mean_mfcc.iter_mut()) {
            *mean += new;
        }
    }

    for i in 0..num_coefficients {
        mean_mfcc[i] /= num_blocks as f32;
    }

    mean_mfcc
}

