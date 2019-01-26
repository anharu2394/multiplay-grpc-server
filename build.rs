extern crate protoc_grpcio;

fn main() {
    let proto_root = "protos";
    let output = "src/protos";
    println!("cargo:rerun-if-changed={}", proto_root);
    protoc_grpcio::compile_grpc_protos(
        &["protos/multiplay.proto"],
        &[proto_root],
        &output
    ).expect("Failed to compile gRPC definitions!");
}
