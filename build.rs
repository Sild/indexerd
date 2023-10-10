extern crate prost_build;

fn main() {
    prost_build::Config::new()
        .out_dir("src/proto")
        .compile_protos(&["proto/req.proto"], &["proto"])
        .unwrap();
}
