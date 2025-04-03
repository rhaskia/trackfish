use ndarray::Array1;
use aubio::{Tempo, FFT, OnsetMode, SpecDesc};

pub fn extract_tempo(buffer: &Vec<f32>, sample_rate: u32) -> f32 {
    let fft_size = 1024;
    let hop_size = 512;
    let num_coefficients = 13;

    let num_blocks = (buffer.len() as f32 / fft_size as f32).floor() as usize;

    let mut fft = FFT::new(fft_size).map_err(|e| e.to_string()).unwrap();
    let mut fft_scratch: Vec<f32> = vec![0.0; fft_size];

    let mut spec = Tempo::new(OnsetMode::Energy, fft_size, hop_size, sample_rate as u32).unwrap();
    let mut mean_spec: Vec<f32> = vec![0.0; num_coefficients];
    let mut spec_scratch: Vec<f32> = vec![0.0; num_coefficients];
    let mut spec_vec = vec![];

    for block_index in 0..num_blocks {
        let start = block_index * fft_size;
        let buf = &buffer[start..];

        fft.do_(buf, &mut fft_scratch).unwrap();
        spec.do_(&fft_scratch, &mut spec_scratch).unwrap();

        spec_vec.push(spec_scratch[0]);
        for (new, mean) in spec_scratch.iter().zip(mean_spec.iter_mut()) {
            *mean += new;
        }
    }

    for i in 0..num_coefficients {
        mean_spec[i] /= num_blocks as f32;
    }

    mean_spec[0]
}
