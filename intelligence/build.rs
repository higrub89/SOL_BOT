use std::env;
use std::path::Path;

fn main() {
    // 🔍 Diagnóstico e Inyección de Fallback de Rutas Críticas
    let protoc = env::var("PROTOC").unwrap_or_else(|_| {
        if Path::new("/usr/bin/protoc").exists() {
            "/usr/bin/protoc".to_string()
        } else {
            "protoc".to_string()
        }
    });

    let protoc_include = env::var("PROTOC_INCLUDE").unwrap_or_else(|_| {
        if Path::new("/usr/include").exists() {
            "/usr/include".to_string()
        } else {
            "/usr/local/include".to_string()
        }
    });

    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PROTOC_INCLUDE");
    println!("cargo:warning=[HFT-AUDIT] PROTOC Detectado: {}", protoc);
    println!(
        "cargo:warning=[HFT-AUDIT] INCLUDE Detectado: {}",
        protoc_include
    );

    // Exportar variables de entorno para prost-build (asegurar propagación interna)
    env::set_var("PROTOC", &protoc);

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../core/proto/signal.proto"],
            &["../core/proto/"]
        )
        .unwrap_or_else(|e| {
            panic!(
                "\n\n🔥 [FALLO CRÍTICO PROTOC — AUDITORÍA REQUIERE INTERVENCIÓN]\nERROR: {:?}\nESTADO: Path={}, Include={}\n",
                e, protoc, protoc_include
            )
        });
}
