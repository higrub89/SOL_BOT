// Build script para compilar archivos .proto a código Rust
// Este script se ejecuta automáticamente antes de cada compilación

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Usar protoc embebido para builds portables (CI/CD)
    std::env::set_var("PROTOC", protobuf_src::protoc());

    // Compilar chassis.proto y geyser.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(&["proto/chassis.proto", "proto/geyser.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/chassis.proto");
    println!("cargo:rerun-if-changed=proto/geyser.proto");
    println!("cargo:rerun-if-changed=proto");

    Ok(())
}
