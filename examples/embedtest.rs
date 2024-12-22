fn main() {
    let encoder = music::embed::AutoEncoder::new().unwrap();

    encoder.encode([0; 1094]);
}
