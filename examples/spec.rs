use trackfish::analysis::{extract_spectral, load_samples};

fn main() {
    let (samples, sample_rate) = load_samples("/mnt/sdcard/music/360 [nI6GP8wKJ6o].mp3", None);

    let spec = extract_spectral(&samples, sample_rate);
    let mean = spec.sum() / spec.len() as f32;
    println!("{spec}");

    std::fs::write("./spec.txt", spec.map(|weight| weight.to_string()).to_vec().join("\n")).unwrap();
}
