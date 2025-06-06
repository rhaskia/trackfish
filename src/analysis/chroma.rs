use ndarray::{Array1, Array2};
use ndarray_npy::ReadNpyExt;
use rustfft::{num_complex::Complex, FftPlanner};

const E_WEIGHTS: &[u8; 49328] = include_bytes!("../../chroma.npy");

pub fn extract_chroma(audio_data: &[f32]) -> Array1<f32> {
    let frame_size = 2048;
    let hop_size = 2048;
    let mut chroma_vectors = Vec::new();
    let chroma_weights = Array2::<f32>::read_npy(&E_WEIGHTS[..]).unwrap();
    let chroma_weights = chroma_weights.t();

    let mut fft_planner = FftPlanner::new();
    let fft = fft_planner.plan_fft_forward(frame_size);

    for frame in audio_data.chunks(hop_size) {
        let _hann_window: Vec<f32> = (0..frame_size)
            .map(|n| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * n as f32 / (frame_size as f32)).cos())
            })
            .collect();

        let hamming_window: Vec<f32> = (0..frame_size)
            .map(|n| {
                0.54 - 0.46 * (2.0 * std::f32::consts::PI * n as f32 / (frame_size as f32)).cos()
            })
            .collect();

        let mut buffer: Vec<Complex<f32>> = frame
            .iter()
            .enumerate()
            .map(|(i, &x)| Complex {
                re: x * hamming_window[i],
                im: 0.0,
            })
            .collect();

        buffer.resize(frame_size, Complex { re: 0.0, im: 0.0 });

        fft.process(&mut buffer);

        let range = buffer[0..1025]
            .into_iter()
            .map(|x| x.norm())
            .collect::<Vec<f32>>();
        let buf = Array1::from_vec(range).insert_axis(ndarray::Axis(0));
        let mut chroma = buf.dot(&chroma_weights);

        let norm: f32 = chroma.sum();
        if norm > 0.0 {
            for value in &mut chroma {
                *value /= norm;
            }
        }

        chroma_vectors.push(chroma.into_shape_with_order((13,)).unwrap());
    }

    let mut mean_chroma = Array1::zeros(13);

    for i in 0..13 {
        for chroma in &chroma_vectors {
            mean_chroma[i] += chroma[i]
        }
        mean_chroma /= chroma_vectors.len() as f32;
    }

    mean_chroma
}
