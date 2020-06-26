fn main() {
    prost_build::compile_protos(
        &["src/discover/proto/message.proto", "src/peer/proto/peer.proto"],
        &["src/"],
    )
    .expect("error compiling .proto files");
}
