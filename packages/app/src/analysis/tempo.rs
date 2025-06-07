use aubio::{OnsetMode, Tempo};

pub fn extract_tempo(buffer: &Vec<f32>, sample_rate: u32) -> f32 {
    let window_size = 1024;
    let hop_size = 512;
    let num_coefficients = 13;

    let num_blocks = (buffer.len() as f32 / window_size as f32).floor() as usize;

    let mut tempo =
        Tempo::new(OnsetMode::Energy, window_size, hop_size, sample_rate as u32).unwrap();
    let mut mean_tempo = 0.0;
    let mut tempo_scratch: Vec<f32> = vec![0.0; num_coefficients];

    for block_index in 0..num_blocks {
        let start = block_index * window_size;
        let buf = &buffer[start..];

        tempo.do_(&buf, &mut tempo_scratch).unwrap();

        mean_tempo += tempo.get_bpm();
    }

    mean_tempo / (num_blocks as f32)
}
