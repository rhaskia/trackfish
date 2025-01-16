use crate::utils::strip_unnessecary;
use ndarray::{Array1, Array2, ArrayBase};
use ndarray_npy::ReadNpyExt;

const E_WEIGHTS: &[u8; 69440] = include_bytes!("../../models/encoder_weights.npy");
const E_BIASES: &[u8; 192] = include_bytes!("../../models/encoder_biases.npy");
const D_WEIGHTS: &[u8; 69440] = include_bytes!("../../models/decoder_weights.npy");
const D_BIASES: &[u8; 4460] = include_bytes!("../../models/decoder_biases.npy");

#[derive(PartialEq)]
pub struct AutoEncoder {
    encoder_weights: Array2<f32>,
    encoder_biases: Array1<f32>,
    decoder_weights: Array2<f32>,
    decoder_biases: Array1<f32>,
    genre_index: Vec<String>
}

impl AutoEncoder {
    pub fn new() -> anyhow::Result<Self> {
        let encoder_weights = Array2::<f32>::read_npy(&E_WEIGHTS[..])?;
        let encoder_biases = Array1::<f32>::read_npy(&E_BIASES[..])?;
        let decoder_weights = Array2::<f32>::read_npy(&D_WEIGHTS[..])?;
        let decoder_biases = Array1::<f32>::read_npy(&D_BIASES[..])?;

        let genre_index = std::fs::read_to_string("./models/genrelist")?.split("\n").map(|genre| genre.trim().to_string()).collect();

        Ok(AutoEncoder {
            genre_index,
            encoder_weights,
            encoder_biases,
            decoder_weights,
            decoder_biases,
        })
    }

    pub fn encode(&self, input: Array1<f32>) -> Array1<f32> {
        assert_eq!(self.encoder_weights.shape()[0], input.len());
        assert_eq!(self.encoder_biases.len(), self.encoder_weights.shape()[1]);
        let mut result = input.dot(&self.encoder_weights) + self.encoder_biases.clone();
        
        // ReLu Activation
        result = result.clamp(0.0, 1_000_000.0);

        result
    }

    pub fn decode(&self, input: Array1<f32>) -> Array1<f32> {
        assert_eq!(self.decoder_weights.shape()[0], input.len());
        assert_eq!(self.decoder_biases.len(), self.decoder_weights.shape()[1]);
        let mut result = input.dot(&self.decoder_weights) + self.decoder_biases.clone();

        // Sigmoid Activation
        result = 1.0 / (1.0 + (result * -1.0).exp());

        result
    }

    // Turns genres into a one-hot encoding before they are encoded further
    pub fn genres_to_vec(&self, genres: Vec<String>) -> Array1<f32> {
        let mut encoding = ArrayBase::from_vec(vec![0.0; self.genre_index.len()]);

        for genre in genres {
            if let Some(idx) = self
                .genre_index
                .iter()
                .position(|other_genre| *other_genre == strip_unnessecary(&genre))
            {
                encoding[idx] = 1.0;
            }
        }

        return encoding;
    }
}
