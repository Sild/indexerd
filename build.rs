extern crate prost_build;
use std::fs;

fn main() {
    fs::create_dir_all("src/proto").unwrap();
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]")
        .out_dir("src/proto")
        .compile_protos(&["proto/search_params.proto"], &["proto"])
        .unwrap();
}
