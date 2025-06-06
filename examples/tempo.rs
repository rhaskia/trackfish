use trackfish::analysis::{extract_tempo, extract_zcr, load_samples};

fn main() {
    let (samples, sample_rate) = load_samples("E:/music/911 [xkDabM0Cy-E].mp3", None);
    println!("{sample_rate}");

    let tempo = extract_tempo(&samples, sample_rate);
    println!("{tempo}");
}
