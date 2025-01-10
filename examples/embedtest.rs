use ndarray::ArrayBase;


fn main() {
    let encoder = music::embed::AutoEncoder::new().unwrap();
    let mut input = vec![0.0; 1083];
    input[3] = 1.0;

    let encoded = encoder.encode(ArrayBase::from_vec(input)).unwrap();
    println!("{:?}", encoded);
    println!("{:?}", encoder.decode(encoded).unwrap());
}
