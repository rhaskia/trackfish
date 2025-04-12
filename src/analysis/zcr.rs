pub fn extract_zcr(buffer: &Vec<f32>, _sample_rate: u32) -> usize {
    let zero_threshold = 0.0001;
    let mut count = 0;
    let mut last_sign = sign(buffer[0]);

    for i in 0..buffer.len() {
        // skip close to zero values
        if buffer[i].abs() < zero_threshold {
            continue;
        }

        if sign(buffer[i]) != last_sign {
            count += 1;
        }

        last_sign = sign(buffer[i]);
    }

    count
}

fn sign(value: f32) -> usize {
    (value / value.abs()) as usize
}
