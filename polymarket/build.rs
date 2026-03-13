// Build script para compilar archivos .proto a código Rust
// Este script se ejecuta automáticamente antes de cada compilación

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compilar polymarket.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/polymarket.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/polymarket.proto");

    Ok(())
}
