fn main() {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(&["../core/proto/signal.proto"], &["../core/proto/"])
        .unwrap();
}
