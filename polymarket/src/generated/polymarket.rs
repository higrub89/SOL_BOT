#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketsRequest {
    /// Filtro opcional por estado del mercado
    #[prost(enumeration = "MarketStatus", tag = "1")]
    pub status_filter: i32,
    /// Límite de resultados (default: 50)
    #[prost(int32, tag = "2")]
    pub limit: i32,
    /// Cursor de paginación
    #[prost(string, tag = "3")]
    pub cursor: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetMarketsResponse {
    #[prost(message, repeated, tag = "1")]
    pub markets: ::prost::alloc::vec::Vec<Market>,
    #[prost(string, tag = "2")]
    pub next_cursor: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Market {
    /// ID único del mercado en Polymarket
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Pregunta / título del mercado (ej: "¿Ganará X las elecciones?")
    #[prost(string, tag = "2")]
    pub question: ::prost::alloc::string::String,
    /// Descripción detallada del evento
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Probabilidad implícita del YES (0.0 - 1.0)
    #[prost(double, tag = "4")]
    pub yes_price: f64,
    /// Probabilidad implícita del NO (0.0 - 1.0)
    #[prost(double, tag = "5")]
    pub no_price: f64,
    /// Volumen total negociado (en USDC)
    #[prost(double, tag = "6")]
    pub volume: f64,
    /// Liquidez disponible (en USDC)
    #[prost(double, tag = "7")]
    pub liquidity: f64,
    /// Fecha de cierre del mercado (Unix timestamp ms)
    #[prost(int64, tag = "8")]
    pub end_date_ms: i64,
    /// Estado actual del mercado
    #[prost(enumeration = "MarketStatus", tag = "9")]
    pub status: i32,
    /// Categoría del evento (ej: "Politics", "Sports", "Crypto")
    #[prost(string, tag = "10")]
    pub category: ::prost::alloc::string::String,
    /// Resultado final si está resuelto
    #[prost(enumeration = "Outcome", tag = "11")]
    pub resolved_outcome: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaceOrderRequest {
    /// ID del mercado donde operar
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Outcome a comprar/vender (YES o NO)
    #[prost(enumeration = "Outcome", tag = "2")]
    pub outcome: i32,
    /// Dirección: compra o venta
    #[prost(enumeration = "Side", tag = "3")]
    pub side: i32,
    /// Cantidad de shares
    #[prost(double, tag = "4")]
    pub amount: f64,
    /// Precio límite (probabilidad, 0.0 - 1.0)
    #[prost(double, tag = "5")]
    pub limit_price: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaceOrderResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub error_message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOrderRequest {
    #[prost(string, tag = "1")]
    pub order_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOrderResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub error_message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPositionsRequest {
    /// Filtro opcional por market_id
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPositionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub positions: ::prost::alloc::vec::Vec<Position>,
    #[prost(double, tag = "2")]
    pub total_pnl: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    /// ID del mercado
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Pregunta del mercado
    #[prost(string, tag = "2")]
    pub question: ::prost::alloc::string::String,
    /// Outcome que se tiene (YES o NO)
    #[prost(enumeration = "Outcome", tag = "3")]
    pub outcome: i32,
    /// Cantidad de shares
    #[prost(double, tag = "4")]
    pub shares: f64,
    /// Precio medio de entrada (probabilidad)
    #[prost(double, tag = "5")]
    pub avg_entry_price: f64,
    /// Precio actual del outcome
    #[prost(double, tag = "6")]
    pub current_price: f64,
    /// PnL no realizado en porcentaje
    #[prost(double, tag = "7")]
    pub pnl_percent: f64,
    /// PnL no realizado en USDC
    #[prost(double, tag = "8")]
    pub pnl_usdc: f64,
    /// Estado del mercado
    #[prost(enumeration = "MarketStatus", tag = "9")]
    pub market_status: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketStreamRequest {
    /// IDs de mercados a seguir (vacío = todos)
    #[prost(string, repeated, tag = "1")]
    pub market_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketUpdate {
    /// ID del mercado actualizado
    #[prost(string, tag = "1")]
    pub market_id: ::prost::alloc::string::String,
    /// Nuevo precio YES
    #[prost(double, tag = "2")]
    pub yes_price: f64,
    /// Nuevo precio NO
    #[prost(double, tag = "3")]
    pub no_price: f64,
    /// Volumen del último trade
    #[prost(double, tag = "4")]
    pub last_trade_volume: f64,
    /// Timestamp de la actualización (Unix ms)
    #[prost(int64, tag = "5")]
    pub timestamp_ms: i64,
    /// Estado del mercado
    #[prost(enumeration = "MarketStatus", tag = "6")]
    pub status: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Side {
    Unspecified = 0,
    Buy = 1,
    Sell = 2,
}
impl Side {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Side::Unspecified => "SIDE_UNSPECIFIED",
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIDE_UNSPECIFIED" => Some(Self::Unspecified),
            "BUY" => Some(Self::Buy),
            "SELL" => Some(Self::Sell),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Outcome {
    Unspecified = 0,
    Yes = 1,
    No = 2,
}
impl Outcome {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Outcome::Unspecified => "OUTCOME_UNSPECIFIED",
            Outcome::Yes => "YES",
            Outcome::No => "NO",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OUTCOME_UNSPECIFIED" => Some(Self::Unspecified),
            "YES" => Some(Self::Yes),
            "NO" => Some(Self::No),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OrderStatus {
    Unspecified = 0,
    Pending = 1,
    Filled = 2,
    PartiallyFilled = 3,
    Cancelled = 4,
    Rejected = 5,
}
impl OrderStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            OrderStatus::Unspecified => "ORDER_STATUS_UNSPECIFIED",
            OrderStatus::Pending => "PENDING",
            OrderStatus::Filled => "FILLED",
            OrderStatus::PartiallyFilled => "PARTIALLY_FILLED",
            OrderStatus::Cancelled => "CANCELLED",
            OrderStatus::Rejected => "REJECTED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ORDER_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "PENDING" => Some(Self::Pending),
            "FILLED" => Some(Self::Filled),
            "PARTIALLY_FILLED" => Some(Self::PartiallyFilled),
            "CANCELLED" => Some(Self::Cancelled),
            "REJECTED" => Some(Self::Rejected),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MarketStatus {
    Unspecified = 0,
    Active = 1,
    Closed = 2,
    Resolved = 3,
}
impl MarketStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MarketStatus::Unspecified => "MARKET_STATUS_UNSPECIFIED",
            MarketStatus::Active => "ACTIVE",
            MarketStatus::Closed => "CLOSED",
            MarketStatus::Resolved => "RESOLVED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MARKET_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "ACTIVE" => Some(Self::Active),
            "CLOSED" => Some(Self::Closed),
            "RESOLVED" => Some(Self::Resolved),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod polymarket_bot_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct PolymarketBotClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PolymarketBotClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> PolymarketBotClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> PolymarketBotClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            PolymarketBotClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Consulta de mercados disponibles
        pub async fn get_markets(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMarketsRequest>,
        ) -> Result<tonic::Response<super::GetMarketsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/polymarket.PolymarketBot/GetMarkets",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Colocar una orden (compra/venta de shares YES/NO)
        pub async fn place_order(
            &mut self,
            request: impl tonic::IntoRequest<super::PlaceOrderRequest>,
        ) -> Result<tonic::Response<super::PlaceOrderResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/polymarket.PolymarketBot/PlaceOrder",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Cancelar una orden activa
        pub async fn cancel_order(
            &mut self,
            request: impl tonic::IntoRequest<super::CancelOrderRequest>,
        ) -> Result<tonic::Response<super::CancelOrderResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/polymarket.PolymarketBot/CancelOrder",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Consultar posiciones abiertas
        pub async fn get_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPositionsRequest>,
        ) -> Result<tonic::Response<super::GetPositionsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/polymarket.PolymarketBot/GetPositions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Stream en tiempo real de actualizaciones de mercado
        pub async fn stream_market_updates(
            &mut self,
            request: impl tonic::IntoRequest<super::MarketStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::MarketUpdate>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/polymarket.PolymarketBot/StreamMarketUpdates",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod polymarket_bot_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PolymarketBotServer.
    #[async_trait]
    pub trait PolymarketBot: Send + Sync + 'static {
        /// Consulta de mercados disponibles
        async fn get_markets(
            &self,
            request: tonic::Request<super::GetMarketsRequest>,
        ) -> Result<tonic::Response<super::GetMarketsResponse>, tonic::Status>;
        /// Colocar una orden (compra/venta de shares YES/NO)
        async fn place_order(
            &self,
            request: tonic::Request<super::PlaceOrderRequest>,
        ) -> Result<tonic::Response<super::PlaceOrderResponse>, tonic::Status>;
        /// Cancelar una orden activa
        async fn cancel_order(
            &self,
            request: tonic::Request<super::CancelOrderRequest>,
        ) -> Result<tonic::Response<super::CancelOrderResponse>, tonic::Status>;
        /// Consultar posiciones abiertas
        async fn get_positions(
            &self,
            request: tonic::Request<super::GetPositionsRequest>,
        ) -> Result<tonic::Response<super::GetPositionsResponse>, tonic::Status>;
        /// Server streaming response type for the StreamMarketUpdates method.
        type StreamMarketUpdatesStream: futures_core::Stream<
                Item = Result<super::MarketUpdate, tonic::Status>,
            >
            + Send
            + 'static;
        /// Stream en tiempo real de actualizaciones de mercado
        async fn stream_market_updates(
            &self,
            request: tonic::Request<super::MarketStreamRequest>,
        ) -> Result<tonic::Response<Self::StreamMarketUpdatesStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct PolymarketBotServer<T: PolymarketBot> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: PolymarketBot> PolymarketBotServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PolymarketBotServer<T>
    where
        T: PolymarketBot,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/polymarket.PolymarketBot/GetMarkets" => {
                    #[allow(non_camel_case_types)]
                    struct GetMarketsSvc<T: PolymarketBot>(pub Arc<T>);
                    impl<
                        T: PolymarketBot,
                    > tonic::server::UnaryService<super::GetMarketsRequest>
                    for GetMarketsSvc<T> {
                        type Response = super::GetMarketsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetMarketsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_markets(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetMarketsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/polymarket.PolymarketBot/PlaceOrder" => {
                    #[allow(non_camel_case_types)]
                    struct PlaceOrderSvc<T: PolymarketBot>(pub Arc<T>);
                    impl<
                        T: PolymarketBot,
                    > tonic::server::UnaryService<super::PlaceOrderRequest>
                    for PlaceOrderSvc<T> {
                        type Response = super::PlaceOrderResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PlaceOrderRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).place_order(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PlaceOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/polymarket.PolymarketBot/CancelOrder" => {
                    #[allow(non_camel_case_types)]
                    struct CancelOrderSvc<T: PolymarketBot>(pub Arc<T>);
                    impl<
                        T: PolymarketBot,
                    > tonic::server::UnaryService<super::CancelOrderRequest>
                    for CancelOrderSvc<T> {
                        type Response = super::CancelOrderResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CancelOrderRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).cancel_order(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CancelOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/polymarket.PolymarketBot/GetPositions" => {
                    #[allow(non_camel_case_types)]
                    struct GetPositionsSvc<T: PolymarketBot>(pub Arc<T>);
                    impl<
                        T: PolymarketBot,
                    > tonic::server::UnaryService<super::GetPositionsRequest>
                    for GetPositionsSvc<T> {
                        type Response = super::GetPositionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPositionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_positions(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPositionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/polymarket.PolymarketBot/StreamMarketUpdates" => {
                    #[allow(non_camel_case_types)]
                    struct StreamMarketUpdatesSvc<T: PolymarketBot>(pub Arc<T>);
                    impl<
                        T: PolymarketBot,
                    > tonic::server::ServerStreamingService<super::MarketStreamRequest>
                    for StreamMarketUpdatesSvc<T> {
                        type Response = super::MarketUpdate;
                        type ResponseStream = T::StreamMarketUpdatesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MarketStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).stream_market_updates(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StreamMarketUpdatesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: PolymarketBot> Clone for PolymarketBotServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: PolymarketBot> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: PolymarketBot> tonic::server::NamedService for PolymarketBotServer<T> {
        const NAME: &'static str = "polymarket.PolymarketBot";
    }
}
