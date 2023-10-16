extern crate prost_build;
use std::fs;

fn main() {
    fs::create_dir_all("src/proto").unwrap();
    prost_build::Config::new()
        .out_dir("src/proto")
        .compile_protos(&["proto/req.proto"], &["proto"])
        .unwrap();
}
