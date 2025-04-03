use ndarray::Array1;
use log::info;

pub fn cosine_similarity(a: Array1<f32>, b: Array1<f32>) -> f32 {
    let lower = (a.pow2().sum().sqrt() * b.pow2().sum().sqrt());
    if lower == 0.0 { return 0.0 };
    (a.clone() * b.clone()).sum() / lower
}
