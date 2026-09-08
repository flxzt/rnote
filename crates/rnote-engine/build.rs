use burn_onnx::{LoadStrategy, ModelGen};
use std::env;
fn main() {
    println!("cargo:rerun-if-changed=../../models/student_model.onnx");
    println!("cargo:rerun-if-changed=../../models/student_model.onnx.data");

    let out_dir = env::var("OUT_DIR").unwrap();
    let model_dir = format!("{}/model", out_dir);

    ModelGen::new()
        .input("../../models/student_model.onnx")
        .out_dir(&model_dir)
        .load_strategy(LoadStrategy::Embedded) // TODO FUTURE PROOF AND DO NOT EMBED MODEL IN BINARY
        .run_from_script();
}
