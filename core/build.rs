// Build script para compilar archivos .proto a código Rust
// Este script se ejecuta automáticamente antes de cada compilación

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Usar protoc del sistema si está disponible (CI/CD con protobuf-compiler instalado),
    // de lo contrario compilar uno embebido via protobuf-src.
    // El env var PROTOC se establece en el CI workflow o puede estar en el PATH del sistema.
    if std::env::var("PROTOC").is_err() {
        // Solo compilar protoc embebido si no hay uno del sistema disponible
        std::env::set_var("PROTOC", protobuf_src::protoc());
    }

    // Compilar chassis.proto, geyser.proto y signal.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("src/generated")
        .compile_protos(
            &[
                "proto/chassis.proto",
                "proto/geyser.proto",
                "proto/signal.proto",
            ],
            &["proto"],
        )?;

    // Formatear archivos generados con rustfmt para mantener consistencia con
    // `cargo fmt --check` en CI.
    format_generated_files(&[
        "src/generated/chassis.rs",
        "src/generated/geyser.rs",
        "src/generated/signal.rs",
    ]);

    println!("cargo:rerun-if-changed=proto/chassis.proto");
    println!("cargo:rerun-if-changed=proto/geyser.proto");
    println!("cargo:rerun-if-changed=proto/signal.proto");
    println!("cargo:rerun-if-changed=proto");

    Ok(())
}

/// Ejecuta `rustfmt` sobre los archivos generados para garantizar que `cargo fmt --check` pase.
fn format_generated_files(files: &[&str]) {
    for file in files {
        // Solo formatear si el archivo existe
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
