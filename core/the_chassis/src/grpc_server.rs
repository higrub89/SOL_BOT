use tonic::{transport::Server, Request, Response, Status};
// Importar los tipos generados por tonic-build
pub mod chassis_proto {
    tonic::include_proto!("chassis");
}

use chassis_proto::chassis_service_server::{ChassisService, ChassisServiceServer};
use chassis_proto::{AuditRequest, AuditResponse, Empty, PositionsResponse, TradeRequest, TradeResponse};

#[derive(Debug, Default)]
pub struct MyChassisService;

#[tonic::async_trait]
impl ChassisService for MyChassisService {
    async fn get_positions(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<PositionsResponse>, Status> {
        println!("Got a request: {:?}", request);

        // TODO: Conectar con la base de datos real
        let reply = PositionsResponse {
            positions: vec![], // Placeholder
        };

        Ok(Response::new(reply))
    }

    async fn execute_trade(
        &self,
        request: Request<TradeRequest>,
    ) -> Result<Response<TradeResponse>, Status> {
        println!("Got a trade request: {:?}", request);
        
        let req = request.into_inner();
        
        // Aquí conectaremos con el Executor Trait
        // Por ahora devolvemos éxito simulado
        
        let reply = TradeResponse {
            success: true,
            signature: "simulated_sig".to_string(),
            error_message: "".to_string(),
        };

        Ok(Response::new(reply))
    }
    
    async fn get_token_audit(
        &self,
        request: Request<AuditRequest>,
    ) -> Result<Response<AuditResponse>, Status> {
        // Este endpoint es para que Python reporte auditorías a Rust?
        // O Rust a Python?
        // Según blueprint: Rust -> Python (Client). Python -> Rust (Server)?
        // Re-leyendo blueprint: Rust actúa como CLIENTE gRPC consultando a Python (Server).
        // Pero el proto define un servicio. 
        // Si Rust es el CLIENTE, entonces Python debe implementar el servidor.
        
        // CORRECCIÓN: El roadmap dice "Rust envía mint a Python vía gRPC".
        // Entonces Rust es el CLIENTE y Python el SERVIDOR.
        // Este archivo grpc_server.rs sería si Rust fuera el servidor.
        
        // Dejaremos esto como servidor para permitir que Python envíe comandos a Rust también (bidireccional).
        
        Ok(Response::new(AuditResponse {
            verdict: "SAFE".to_string(),
            score: 85,
            lp_locked_pct: 99.0,
        }))
    }
}

pub async fn start_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyChassisService::default();

    println!("Chassis gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(ChassisServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
