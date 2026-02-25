//! # Yellowstone Geyser gRPC Client (Real Implementation)
//!
//! Cliente para recibir streaming de Account Updates desde Solana.
//! Proporciona ventaja competitiva de baja latencia (<200ms).

use crate::generated::geyser::{
    geyser_client::GeyserClient as ProtoClient, SubscribeRequest, SubscribeRequestFilterAccounts,
};
use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::transport::Channel;
/// Helper para obtener amount de un SPL Token Account data
pub fn parse_spl_token_amount(data: &[u8]) -> Option<u64> {
    if data.len() < 165 {
        // Tamaño mínimo de SPL Token Account
        return None;
    }
    // El campo amount está en el offset 64 (después de mint(32) + owner(32))
    // Leemos 8 bytes como u64 little-endian
    data.get(64..72)
        .map(|slice| u64::from_le_bytes(slice.try_into().unwrap()))
}

/// Configuración del cliente Geyser
pub struct GeyserConfig {
    pub endpoint: String,
    pub token: Option<String>,
}

impl Default for GeyserConfig {
    fn default() -> Self {
        let token = std::env::var("HELIUS_API_KEY").ok();
        Self {
            endpoint: "https://mainnet.helius-rpc.com".to_string(),
            token,
        }
    }
}

/// Interceptor para autenticación de Helius
#[derive(Clone)]
pub struct GeyserAuthInterceptor {
    token: String,
}

impl Interceptor for GeyserAuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        if !self.token.is_empty() {
            // Helius requiere 'x-token' en headers
            let token_meta = MetadataValue::try_from(&self.token)
                .map_err(|_| tonic::Status::invalid_argument("Invalid auth token format"))?;

            request.metadata_mut().insert("x-token", token_meta.clone());

            // Opcional: especificar encoding
            // request.metadata_mut().insert("content-encoding", "gzip".parse().unwrap());
        }
        Ok(request)
    }
}

/// Cliente real de Yellowstone Geyser
pub struct GeyserClient {
    config: GeyserConfig,
}

impl GeyserClient {
    pub fn new(config: GeyserConfig) -> Self {
        Self { config }
    }

    /// Conecta al servidor gRPC con autenticación
    pub async fn connect(
        &self,
    ) -> Result<
        ProtoClient<
            tonic::service::interceptor::InterceptedService<Channel, GeyserAuthInterceptor>,
        >,
    > {
        // println!("🔌 Conectando a Yellowstone Geyser (gRPC)...");
        // println!("   Endpoint: {}", self.config.endpoint);

        let endpoint = tonic::transport::Endpoint::from_shared(self.config.endpoint.clone())?
            .tls_config(tonic::transport::ClientTlsConfig::new())?
            .keep_alive_timeout(Duration::from_secs(10))
            .keep_alive_while_idle(true)
            .connect_timeout(Duration::from_secs(5));

        let channel = endpoint.connect().await
            .context("Error al conectar con el servidor gRPC de Geyser. Verifica tu HELIUS_API_KEY y conexión.")?;

        let token = self.config.token.clone().unwrap_or_default();
        let interceptor = GeyserAuthInterceptor { token };

        // println!("✅ Conexión establecida gRPC (TLS + Auth)");

        Ok(ProtoClient::with_interceptor(channel, interceptor))
    }

    /// Suscribe a actualizaciones de una cuenta y maneja el stream
    pub async fn subscribe_and_listen(&self, pubkey: &str) -> Result<()> {
        println!("🔌 Inicializando conexión Geyser...");
        let mut client = self.connect().await?;

        // Creamos canal para enviar peticiones de subscripción (bidireccional)
        let (tx, rx) = mpsc::channel(100);

        // Request inicial
        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            "client_subscription".to_string(),
            SubscribeRequestFilterAccounts {
                account: vec![pubkey.to_string()],
                owner: vec![],
                filters: vec![],
            },
        );

        let request = SubscribeRequest {
            accounts,
            slots: std::collections::HashMap::new(),
            transactions: std::collections::HashMap::new(),
            blocks: std::collections::HashMap::new(),
            blocks_meta: std::collections::HashMap::new(),
            entry: None,
            commitment: Some(0), // PROCESSED (Baja latencia)
            accounts_data_slice: std::collections::HashMap::new(),
            ping: None,
        };

        println!("📤 Enviando solicitud de suscripción para: {}", pubkey);
        tx.send(request).await?;

        // Iniciamos el stream
        let stream_result = client.subscribe(ReceiverStream::new(rx)).await;

        match stream_result {
            Ok(response) => {
                let mut response_stream = response.into_inner();
                println!("✅ STREAM ACTIVO recibiendo datos de: {}", pubkey);

                // Bucle de escucha de eventos
                // Bloquea la task actual
                while let Ok(Some(update)) = response_stream.message().await {
                    if let Some(event) = update.update_oneof {
                        match event {
                            crate::generated::geyser::subscribe_update::UpdateOneof::Account(
                                acc_update,
                            ) => {
                                if let Some(info) = acc_update.account {
                                    println!(
                                        "⚡ UPDATE CUENTA | Slot: {} | Data Len: {} bytes",
                                        acc_update.slot,
                                        info.data.len()
                                    );
                                }
                            }
                            crate::generated::geyser::subscribe_update::UpdateOneof::Ping(_) => {
                                // Ignore pings logging
                            }
                            _ => println!("📝 Otro evento: {:?}", event),
                        }
                    }
                }
                println!("🛑 Stream de Geyser cerrado por el servidor");
            }
            Err(e) => {
                eprintln!("❌ Error al iniciar suscripción: {:?}", e);
                return Err(anyhow::anyhow!("Subscription failed: {}", e));
            }
        }

        Ok(())
    }

    /// Benchmark real de latencia gRPC
    pub async fn benchmark_latency(&self) -> Result<u128> {
        let start = Instant::now();
        // Solo conectamos, el handshake TLS + TCP cuenta
        let _ = self.connect().await?;
        let latency = start.elapsed().as_millis();

        // println!("📊 Latencia gRPC handshake: {} ms", latency);
        Ok(latency)
    }
}
