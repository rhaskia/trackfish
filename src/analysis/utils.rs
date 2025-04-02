use ndarray::Array1;
use log::info;

pub fn cosine_similarity(a: Array1<f32>, b: Array1<f32>) -> f32 {
    (a.clone() * b.clone()).sum() / (a.pow2().sum().sqrt() * b.pow2().sum().sqrt()) 
}
