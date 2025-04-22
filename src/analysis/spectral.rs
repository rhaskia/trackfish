use aubio::{SpecDesc, SpecShape, FFT};
use ndarray::Array1;

pub fn extract_spectral(buffer: &Vec<f32>, _sample_rate: u32) -> Array1<f32> {
    let fft_size = 1024;
    let num_coefficients = 6;

    let num_blocks = (buffer.len() as f32 / fft_size as f32).floor() as usize;

    let mut fft = FFT::new(fft_size).map_err(|e| e.to_string()).unwrap();
    let mut fft_scratch: Vec<f32> = vec![0.0; fft_size];

    let mut centroid = SpecDesc::new(SpecShape::Centroid, fft_size).unwrap();
    let mut spread = SpecDesc::new(SpecShape::Spread, fft_size).unwrap();
    let mut skew = SpecDesc::new(SpecShape::Skewness, fft_size).unwrap();
    let mut slope = SpecDesc::new(SpecShape::Slope, fft_size).unwrap();
    let mut decrease = SpecDesc::new(SpecShape::Decrease, fft_size).unwrap();
    let mut rolloff = SpecDesc::new(SpecShape::Rolloff, fft_size).unwrap();

    let mut mean_spec: Vec<f32> = vec![0.0; num_coefficients];
    let mut spec_scratch: Vec<f32> = vec![0.0; num_coefficients];
    let mut spec_vec = vec![];

    for block_index in 0..num_blocks {
        let start = block_index * fft_size;
        let buf = &buffer[start..];

        fft.do_(buf, &mut fft_scratch).unwrap();
        centroid.do_(&fft_scratch, &mut spec_scratch).unwrap();
        spread.do_(&fft_scratch, &mut spec_scratch[1..]).unwrap();
        skew.do_(&fft_scratch, &mut spec_scratch[2..]).unwrap();
        slope.do_(&fft_scratch, &mut spec_scratch[3..]).unwrap();
        decrease.do_(&fft_scratch, &mut spec_scratch[4..]).unwrap();
        rolloff.do_(&fft_scratch, &mut spec_scratch[5..]).unwrap();

        spec_vec.push(spec_scratch[0]);
        for (new, mean) in spec_scratch.iter().zip(mean_spec.iter_mut()) {
            *mean += new;
        }
    }

    for i in 0..num_coefficients {
        mean_spec[i] /= num_blocks as f32;
    }

    Array1::from_vec(mean_spec)
}
