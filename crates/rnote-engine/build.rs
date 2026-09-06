use burn_onnx::{LoadStrategy, ModelGen};
use std::env;
fn main() {
    // Tell Cargo to re-run the build script only if the model file changes
    println!("cargo:rerun-if-changed=../../models/student_model.onnx");
    println!("cargo:rerun-if-changed=../../models/student_model.onnx.data");

    let out_dir = env::var("OUT_DIR").unwrap();
    let model_dir = format!("{}/model", out_dir);

    ModelGen::new()
        // Update this path to wherever your ONNX file actually lives
        // (relative to the crates/rnote-engine/ directory)
        .input("../../models/student_model.onnx")
        .out_dir(&model_dir)
        .load_strategy(LoadStrategy::File)
        .run_from_script();
}
