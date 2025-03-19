
use rodio::{Decoder, Source};
use ndarray::{Array1, Array2, ArrayBase};
use ndarray_npy::ReadNpyExt;
use std::fs::File;
use std::io::BufReader;
use aubio::MFCC;
use aubio::FFT;

fn std_dev(arr: Vec<f32>, mean: f32) -> f32 {
    let sum_of_squares = arr.iter().map(|x| (x - mean).powi(2)).sum::<f32>();
    (sum_of_squares / arr.len() as f32).sqrt()
}

use plotters::prelude::*;

const OUT_FILE_NAME: &str = "matshow.png";

fn load_samples(file: &str) -> (Vec<f32>, u32) {
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

    // if channels == 2 {
    //     samples.chunks_exact(2).map(|s| (s[0] + s[1]) / 2.0).collect()
    // } else {
    //     samples.clone()
    // }
    (samples, sample_rate)
}

fn linear_resample(audio_data: &Vec<f32>, input_rate: usize, output_rate: usize) -> Vec<f32> {
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


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let file = File::open("E:\\Music\\Lemon [H5ZPCcnLXt4].mp3");
    let (samples, sample_rate) = load_samples("/home/rhaskia/Downloads/octave.wav");
    //let samples = linear_resample(&samples, sample_rate as usize, 44100);
    println!("{}", samples.len());

    let frame_size = 1024;
    let mfcc_len = 13;

    // let result = calculate_mfcc(&mut mono_samples, sample_rate);
    // let mean = result.iter().sum::<f32>() / mfcc_len as f32;
    // let std = std_dev(result.clone(), mean);
    // let result = result.iter().map(|n| (n - mean) / std).collect::<Vec<f32>>();
    // println!("{result:?}");

    let chroma_vectors = extract_chroma(&samples, sample_rate as usize);
    // let mean = result.iter().sum::<f32>() / 12 as f32;
    // let std = std_dev(result.clone(), mean);
    // let result = result.iter().map(|n| (n - mean) / std).collect::<Vec<f32>>();
    // let chroma_vectors: Vec<Vec<f32>> = chroma_vectors.into_iter().map(|v| { let mut v = v; v.rotate_right(0); v }).collect();

    let mut mean_chroma: Vec<f32> = vec![0.0; 12];

    for i in 0..12 {
        for j in 0..chroma_vectors.len() {
            mean_chroma[i] += chroma_vectors[j][i];
        }

        mean_chroma[i] /= chroma_vectors.len() as f32;
    }
    println!("{mean_chroma:?}");

    let root = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;
    
    let len = chroma_vectors.len();

    let mut chart = ChartBuilder::on(&root)
        .margin(5)
        .top_x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0i32..len as i32, 0i32..12i32)?;

    chart
        .configure_mesh()
        .x_labels(15)
        .max_light_lines(4)
        // .x_label_offset(35)
        // .y_label_offset(25)
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", 20))
        .draw()?;

    chart.draw_series(
        chroma_vectors
            .iter()
            .zip(0..)
            .flat_map(|(l, y)| l.iter().zip(0..).map(move |(v, x)| (y, x, v)))
            .map(|(x, y, v)| {
                Rectangle::new(
                    [(x, y), (x + 1, y + 1)],
                    val_to_hsl(*v)
                    .filled(),
                )
            }),
    )?;

    // let pitches = vec![("B", 40), ("A", 70), ("L", 100)];
    let pitches = vec![("B", 65), ("A", 190), ("G", 305), ("F", 420), ("E", 485), ("D", 605), ("C", 720)];
    let style = TextStyle::from(("sans-serif", 20).into_font());

    for (pitch, height) in pitches {
        root.draw_text(
            pitch,
            &style,
            (20, height),
        );
    }

    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}

fn val_to_hsl(v: f32) -> HSLColor {
    let v = (v * 6.0) - 0.1;
    let hue = if v > 0.5 {
        0.0
    } else {
        0.6
    };

    let value = if v > 0.5 {
        1.0 - ((v as f64 - 0.5))
    } else {
        v as f64 * 2.0
    };

    HSLColor(
        hue,
        0.8,
        value
    )
}

use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

const E_WEIGHTS: &[u8; 49328] = include_bytes!("../chroma.npy");

fn extract_chroma(audio_data: &[f32], sample_rate: usize) -> Vec<Vec<f32>> {
    let frame_size = 2048;
    let num_coefficients = 12;
    let hop_size = 2048;
    let mut chroma_vectors = Vec::new();
    let chroma_weights = Array2::<f32>::read_npy(&E_WEIGHTS[..]).unwrap();
    let chroma_weights = chroma_weights.t();

    let mut fft_planner = FftPlanner::new();
    let fft = fft_planner.plan_fft_forward(frame_size);

    for frame in audio_data.chunks(hop_size) {
        let hann_window: Vec<f32> = (0..frame_size)
            .map(|n| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * n as f32 / (frame_size as f32)).cos()))
            .collect();

        let hamming_window: Vec<f32> = (0..frame_size)
            .map(|n| 0.54 - 0.46 * (2.0 * std::f32::consts::PI * n as f32 / (frame_size as f32)).cos())
            .collect();

        let mut buffer: Vec<Complex<f32>> = frame.iter()
            .enumerate()
            .map(|(i, &x)| Complex { re: x * hamming_window[i], im: 0.0 })
            .collect();

        for i in 800..buffer.len() {
            //buffer[i] = Complex { re: 0.0, im: 0.0 };
        }

        buffer.resize(frame_size, Complex { re: 0.0, im: 0.0 });

        fft.process(&mut buffer);

        let mut chroma = [0.0; 12];

        let range = buffer[0..1025].into_iter().map(|x| x.norm()).collect::<Vec<f32>>();
        let mut buf = Array1::from_vec(range).insert_axis(ndarray::Axis(0));
        //buf += 1.0;
        let mut chroma = buf.dot(&chroma_weights);

        let norm: f32 = chroma.sum();
        if norm > 0.0 {
            for value in &mut chroma {
                *value /= norm;
            }
        }

        chroma_vectors.push(chroma.into_raw_vec());
    }

    chroma_vectors
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

