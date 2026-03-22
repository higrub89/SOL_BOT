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

    // Formatear archivos generados con rustfmt para mantener consistencia con
    // `cargo fmt --check` en CI. tonic-build 0.12 no expone .format() en Builder,
    // así que lo hacemos explícitamente aquí.
    format_generated_files(&["src/generated/chassis.rs", "src/generated/geyser.rs"]);

    println!("cargo:rerun-if-changed=proto/chassis.proto");
    println!("cargo:rerun-if-changed=proto/geyser.proto");
    println!("cargo:rerun-if-changed=proto");

    Ok(())
}

/// Ejecuta `rustfmt` sobre los archivos generados para garantizar que `cargo fmt --check` pase.
fn format_generated_files(files: &[&str]) {
    for file in files {
        // Solo formatear si el archivo existe (geyser.rs puede no existir en todos los perfiles)
        if std::path::Path::new(file).exists() {
            let status = std::process::Command::new("rustfmt")
                .arg("--edition=2021")
                .arg(file)
                .status();
            // No hacer fallar el build si rustfmt no está disponible (ej. minimal toolchain)
            if let Err(e) = status {
                eprintln!("cargo:warning=rustfmt no disponible, omitiendo formato de {file}: {e}");
            }
        }
    }
}
