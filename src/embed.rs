use crate::track::strip_unnessecary;
use std::collections::HashMap;
use tensorflow::{
    FetchToken, Graph, MetaGraphDef, Operation, SavedModelBundle, Session, SessionOptions,
    SessionRunArgs, SignatureDef, Tensor, TensorInfo,
};

pub struct AutoEncoder {
    graph: Graph,
    model_bundle: SavedModelBundle,
    genre_index: Vec<String>,
}

impl AutoEncoder {
    pub fn new() -> anyhow::Result<Self> {
        let model_path = "./models/encoder16/";

        // Build a new graph
        let mut graph = Graph::new();

        // Load the saved model
        let model_bundle: SavedModelBundle =
            SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, model_path)
                .unwrap();

        let genre_index = std::fs::read_to_string("./models/genrelist")?.split("\n").map(ToString::to_string).collect();

        Ok(AutoEncoder {
            graph,
            model_bundle,
            genre_index
        })
    }

    pub fn encode(&self, input: [f32; 1094]) -> anyhow::Result<()> {
        let graph_definition: &MetaGraphDef = self.model_bundle.meta_graph_def();
        let serv_default: &SignatureDef = graph_definition.get_signature("serving_default")?;

        let sign_def: &HashMap<String, SignatureDef> = graph_definition.signatures();

        let model_input: &TensorInfo = serv_default.get_input("keras_tensor")?;
        let model_output: &TensorInfo = serv_default.get_output("output_0")?;

        let input_op: Operation = self.graph.operation_by_name_required(&model_input.name().name)?;
        let output_op: Operation = self.graph.operation_by_name_required(&model_output.name().name)?;
        let model_input_index = model_input.name().index;
        let model_output_index = model_output.name().index;

        let input_tensor: Tensor<f32> = Tensor::new(&[1, 1094]).with_values(&input)?;

        let mut steps: SessionRunArgs = SessionRunArgs::new();
        steps.add_feed(&input_op, model_input_index, &input_tensor);
        let output_fetch: FetchToken = steps.request_fetch(&output_op, model_output_index);
        let session = &self.model_bundle.session;
        session.run(&mut steps)?;

        let output: Tensor<f32> = steps.fetch::<f32>(output_fetch)?;
        println!("OUTPUT: {:#?}", output);

        Ok(())
    }

    // Turns genres into a one-hot encoding before they are encoded further
    pub fn genres_to_vec(&self, genres: Vec<String>) -> [f32; 1094] {
        let mut encoding = [0.0; 1094];

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
