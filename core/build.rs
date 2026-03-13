// Build script para compilar archivos .proto a código Rust
// Este script se ejecuta automáticamente antes de cada compilación

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the vendored protoc binary to avoid system dependencies
    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", protoc_path);

    // Compilar chassis.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/chassis.proto", "proto/geyser.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/chassis.proto");
    println!("cargo:rerun-if-changed=proto/geyser.proto");

    Ok(())
}
