use std::collections::HashMap;
use tensorflow::{
    Graph, MetaGraphDef, SavedModelBundle, Session, SessionOptions, SessionRunArgs, SignatureDef,
    Tensor, TensorInfo, Operation, FetchToken
};

pub struct AutoEncoder {}

pub fn embed() -> anyhow::Result<()> {
    let model_path = "E:/rust/music/models";

    // Build a new graph
    let mut graph = Graph::new();

    // Load the saved model
    let model_bundle: SavedModelBundle =
        SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, model_path).unwrap();

    let graph_definition: &MetaGraphDef = model_bundle.meta_graph_def();
    let serv_default: &SignatureDef = graph_definition.get_signature("serving_default")?;

    let sign_def: &HashMap<String, SignatureDef> = graph_definition.signatures();
    println!("SIGNATURES: {:#?}", sign_def);

    let model_input: &TensorInfo = serv_default.get_input("keras_tensor")?;
    let model_input_index = model_input.name().index;

    let model_output: &TensorInfo = serv_default.get_output("output_0")?;
    let model_output_index = model_output.name().index;
    println!("OUTPUT: {:#?}", model_output);

    let input_op: Operation =
       graph.operation_by_name_required(&model_input.name().name)?;
    let output_op: Operation =
       graph.operation_by_name_required(&model_output.name().name)?;

    let mut in_values: [f32; 1094] = [0.0; 1094];
    let input_tensor: Tensor<f32> = Tensor::new(&[1, 1094]).with_values(&in_values)?;

    let mut steps: SessionRunArgs = SessionRunArgs::new();
    steps.add_feed(&input_op, model_input_index, &input_tensor);
    let output_fetch: FetchToken = steps.request_fetch(&output_op,
              model_output_index);
    let session: Session = model_bundle.session;
    session.run(&mut steps)?;

    let output: Tensor<f32> = steps.fetch::<f32>(output_fetch)?;
    println!("OUTPUT SHAPE: {}", output.shape());
    println!("OUTPUT: {:#?}", output);

    Ok(())
}
