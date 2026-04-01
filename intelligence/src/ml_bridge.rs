//! # ML Bridge - High Performance gRPC over UDS
//!
//! Conexión de ultrabaja latencia (<0.5ms) entre los modelos Python y el motor Rust.
//! Implementa el servicio gRPC `SignalService` sobre Unix Domain Sockets.

use anyhow::{Context, Result};
use std::path::Path;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status, Streaming};

// Protos generados por tonic-build
pub mod pb {
    tonic::include_proto!("signal");
}

pub use pb::Signal;

use pb::signal_service_server::{SignalService, SignalServiceServer};
use pb::{Heartbeat, SignalAck, SignalStatus};

pub struct MlSignalHandler {
    signal_tx: broadcast::Sender<Signal>,
}

#[tonic::async_trait]
impl SignalService for MlSignalHandler {
    async fn stream_signals(
        &self,
        request: Request<Streaming<Signal>>,
    ) -> Result<Response<SignalAck>, Status> {
        let mut stream = request.into_inner();
        let tx = self.signal_tx.clone();

        // Procesar stream en el hot-path (Worker pre-asignado por Tonic)
        tokio::spawn(async move {
            while let Some(signal) = stream.message().await.unwrap_or(None) {
                // Broadcast al motor de ruteo
                let _ = tx.send(signal);
            }
        });

        Ok(Response::new(SignalAck {
            signal_id: "STREAM_SESSION".to_string(),
            status: SignalStatus::Pending as i32,
            tx_sig: String::new(),
            slot: 0,
            ack_ts_ms: chrono::Utc::now().timestamp_millis() as u64,
            reject_reason: String::new(),
        }))
    }

    async fn send_heartbeat(
        &self,
        request: Request<Heartbeat>,
    ) -> Result<Response<SignalAck>, Status> {
        let hb = request.into_inner();
        println!(
            "💓 [ML-BRIDGE] Heartbeat v{} from {}",
            hb.model_version, hb.process_id
        );

        Ok(Response::new(SignalAck {
            signal_id: format!("HB_{}", hb.timestamp_ms),
            status: SignalStatus::Pending as i32,
            ..Default::default()
        }))
    }
}

pub struct MlBridge {
    socket_path: String,
    signal_tx: broadcast::Sender<Signal>,
}

impl MlBridge {
    pub fn new(socket_path: &str) -> (Self, broadcast::Receiver<Signal>) {
        let (tx, rx) = broadcast::channel(4096); // Buffer grande para picos HFT
        (
            Self {
                socket_path: socket_path.to_string(),
                signal_tx: tx,
            },
            rx,
        )
    }

    pub async fn run(self) -> Result<()> {
        let path = Path::new(&self.socket_path);

        if path.exists() {
            std::fs::remove_file(path).context("No se pudo limpiar socket previo")?;
        }

        let uds = UnixListener::bind(path).context("Fallo al bindear socket UDS")?;
        let uds_stream = UnixListenerStream::new(uds);

        let handler = MlSignalHandler {
            signal_tx: self.signal_tx,
        };

        println!(
            "🧠 [ML-BRIDGE] Servidor gRPC (UDS) Activo: {}",
            self.socket_path
        );

        Server::builder()
            .add_service(SignalServiceServer::new(handler))
            .serve_with_incoming(uds_stream)
            .await
            .context("Error fatal en el servidor gRPC ML Bridge")?;

        Ok(())
    }
}
