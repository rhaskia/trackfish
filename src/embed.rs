use tensorflow::SavedModelBundle;
use tensorflow::SessionOptions;
use tensorflow::Graph;
use tensorflow::SessionRunArgs;
use tensorflow::Tensor;
use tensorflow::Session;

pub struct AutoEncoder {

}

pub fn embed() -> anyhow::Result<()> {
    let model_path = "E:/rust/music/models"; 

    // Build a new graph
    let mut graph = Graph::new();

    // Load the saved model
    let model_bundle: SavedModelBundle = SavedModelBundle::load(
        &SessionOptions::new(), 
        &["serve"],
        &mut graph, 
        model_path,
    ).unwrap();

    let graph_definition: &MetaGraphDef = model_bundle.meta_graph_def();
    let serv_default: &SignatureDef = 
            graph_definition.get_signature("serving_default")?;

    let mut session = Session::new(&SessionOptions::new(), &graph)?;

    let input_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let tensor1 = Tensor::new(&[1, input_data.len() as u64]).with_values(&input_data)?;

    let input_operation = graph.operation_by_name("serving_default_input_1")?; 
    let output_operation = graph.operation_by_name("StatefulPartitionedCall")?; 

    // Run the encoder
    let mut args = SessionRunArgs::new();
    args.add_feed(&op1, 0, &tensor1);
    let result_token = args.request_fetch(&op3, 0);
    session.run(&mut args)?;
    let result_tensor = args.fetch(result_token)?;

    println!("Encoded values: {:?}", result_tensor);

    Ok(())
}
