use ndarray::Array1;
use aubio::{MFCC, FFT};

pub fn extract_mfcc(buffer: &Vec<f32>, sample_rate: u32) -> Array1<f32> {
    let fft_size = 1024;
    let num_coefficients = 13;
    let num_filters = 20;

    let num_blocks = (buffer.len() as f32 / fft_size as f32).floor() as usize;
    if num_blocks == 0 {
        // buffer.resize(fft_size, 0.0);
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

    Array1::from_vec(mean_mfcc)
}

