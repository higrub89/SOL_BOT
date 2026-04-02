use std::env;

fn main() {
    // 🔍 Diagnóstico de Entorno
    let protoc = env::var("PROTOC").unwrap_or_else(|_| "protoc".to_string());
    let protoc_include = env::var("PROTOC_INCLUDE").unwrap_or_else(|_| "/usr/include".to_string());

    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=PROTOC_INCLUDE");
    println!("cargo:warning=[HFT] Usando PROTOC={}", protoc);
    println!(
        "cargo:warning=[HFT] Usando PROTOC_INCLUDE={}",
        protoc_include
    );

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &["../core/proto/signal.proto"],
            &["../core/proto/"]
        )
        .unwrap_or_else(|e| {
            panic!(
                "\n\n🔥 [FALLO CRÍTICO PROTOC]\nNo se pudieron compilar los protocolos.\nError: {:?}\nPROTOC: {}\nPROTOC_INCLUDE: {}\n\n",
                e, protoc, protoc_include
            )
        });
}
