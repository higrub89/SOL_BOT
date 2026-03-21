#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequest {
    #[prost(map = "string, message", tag = "1")]
    pub accounts:
        ::std::collections::HashMap<::prost::alloc::string::String, SubscribeRequestFilterAccounts>,
    #[prost(map = "string, message", tag = "2")]
    pub slots:
        ::std::collections::HashMap<::prost::alloc::string::String, SubscribeRequestFilterSlots>,
    #[prost(map = "string, message", tag = "3")]
    pub transactions: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        SubscribeRequestFilterTransactions,
    >,
    #[prost(map = "string, message", tag = "4")]
    pub blocks:
        ::std::collections::HashMap<::prost::alloc::string::String, SubscribeRequestFilterBlocks>,
    #[prost(map = "string, message", tag = "5")]
    pub blocks_meta: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        SubscribeRequestFilterBlocksMeta,
    >,
    #[prost(message, optional, tag = "6")]
    pub entry: ::core::option::Option<SubscribeRequestFilterEntry>,
    #[prost(enumeration = "CommitmentLevel", optional, tag = "7")]
    pub commitment: ::core::option::Option<i32>,
    #[prost(map = "string, message", tag = "8")]
    pub accounts_data_slice: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        SubscribeRequestFilterAccountsDataSlice,
    >,
    #[prost(uint64, optional, tag = "9")]
    pub ping: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterAccounts {
    #[prost(string, repeated, tag = "1")]
    pub account: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "2")]
    pub owner: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "3")]
    pub filters: ::prost::alloc::vec::Vec<SubscribeRequestFilterAccountsFilter>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterAccountsFilter {
    #[prost(
        oneof = "subscribe_request_filter_accounts_filter::Filter",
        tags = "1, 2"
    )]
    pub filter: ::core::option::Option<subscribe_request_filter_accounts_filter::Filter>,
}
/// Nested message and enum types in `SubscribeRequestFilterAccountsFilter`.
pub mod subscribe_request_filter_accounts_filter {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Filter {
        #[prost(message, tag = "1")]
        Memcmp(super::SubscribeRequestFilterAccountsFilterMemcmp),
        #[prost(uint64, tag = "2")]
        DataSize(u64),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterAccountsFilterMemcmp {
    #[prost(uint64, tag = "1")]
    pub offset: u64,
    #[prost(
        oneof = "subscribe_request_filter_accounts_filter_memcmp::Bytes",
        tags = "2, 3"
    )]
    pub bytes: ::core::option::Option<subscribe_request_filter_accounts_filter_memcmp::Bytes>,
}
/// Nested message and enum types in `SubscribeRequestFilterAccountsFilterMemcmp`.
pub mod subscribe_request_filter_accounts_filter_memcmp {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Bytes {
        #[prost(string, tag = "2")]
        Base58(::prost::alloc::string::String),
        #[prost(bytes, tag = "3")]
        Raw(::prost::alloc::vec::Vec<u8>),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterAccountsDataSlice {
    #[prost(uint64, tag = "1")]
    pub offset: u64,
    #[prost(uint64, tag = "2")]
    pub length: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterSlots {
    #[prost(bool, optional, tag = "1")]
    pub filter_by_commitment: ::core::option::Option<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterTransactions {
    #[prost(bool, optional, tag = "1")]
    pub vote: ::core::option::Option<bool>,
    #[prost(bool, optional, tag = "2")]
    pub failed: ::core::option::Option<bool>,
    #[prost(string, optional, tag = "3")]
    pub signature: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "4")]
    pub account_include: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub account_exclude: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "6")]
    pub account_required: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterBlocks {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterBlocksMeta {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeRequestFilterEntry {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdate {
    #[prost(string, repeated, tag = "8")]
    pub filters: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(oneof = "subscribe_update::UpdateOneof", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub update_oneof: ::core::option::Option<subscribe_update::UpdateOneof>,
}
/// Nested message and enum types in `SubscribeUpdate`.
pub mod subscribe_update {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum UpdateOneof {
        #[prost(message, tag = "1")]
        Account(super::SubscribeUpdateAccount),
        #[prost(message, tag = "2")]
        Slot(super::SubscribeUpdateSlot),
        #[prost(message, tag = "3")]
        Transaction(super::SubscribeUpdateTransaction),
        #[prost(message, tag = "4")]
        Block(super::SubscribeUpdateBlock),
        #[prost(message, tag = "5")]
        Ping(super::SubscribeUpdatePing),
        #[prost(message, tag = "6")]
        BlockMeta(super::SubscribeUpdateBlockMeta),
        #[prost(message, tag = "7")]
        Entry(super::SubscribeUpdateEntry),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateAccount {
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<SubscribeUpdateAccountInfo>,
    #[prost(uint64, tag = "2")]
    pub slot: u64,
    #[prost(bool, tag = "3")]
    pub is_startup: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateAccountInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub pubkey: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub lamports: u64,
    #[prost(bytes = "vec", tag = "3")]
    pub owner: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "4")]
    pub executable: bool,
    #[prost(uint64, tag = "5")]
    pub rent_epoch: u64,
    #[prost(bytes = "vec", tag = "6")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "7")]
    pub write_version: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateSlot {
    #[prost(uint64, tag = "1")]
    pub slot: u64,
    #[prost(uint64, optional, tag = "2")]
    pub parent: ::core::option::Option<u64>,
    #[prost(enumeration = "CommitmentLevel", tag = "3")]
    pub status: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateTransaction {
    #[prost(message, optional, tag = "1")]
    pub transaction: ::core::option::Option<SubscribeUpdateTransactionInfo>,
    #[prost(uint64, tag = "2")]
    pub slot: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateTransactionInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    /// Simplified transaction info
    #[prost(bool, tag = "2")]
    pub is_vote: bool,
}
/// Simplified block info
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateBlock {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateBlockMeta {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdateEntry {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeUpdatePing {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingRequest {
    #[prost(int32, tag = "1")]
    pub count: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingResponse {
    #[prost(int32, tag = "1")]
    pub count: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CommitmentLevel {
    Processed = 0,
    Confirmed = 1,
    Finalized = 2,
}
impl CommitmentLevel {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CommitmentLevel::Processed => "PROCESSED",
            CommitmentLevel::Confirmed => "CONFIRMED",
            CommitmentLevel::Finalized => "FINALIZED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PROCESSED" => Some(Self::Processed),
            "CONFIRMED" => Some(Self::Confirmed),
            "FINALIZED" => Some(Self::Finalized),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod geyser_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct GeyserClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GeyserClient<tonic::transport::Channel> {
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
    impl<T> GeyserClient<T>
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
        ) -> GeyserClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            GeyserClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn subscribe(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::SubscribeRequest>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::SubscribeUpdate>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/geyser.Geyser/Subscribe");
            self.inner
                .streaming(request.into_streaming_request(), path, codec)
                .await
        }
        pub async fn ping(
            &mut self,
            request: impl tonic::IntoRequest<super::PingRequest>,
        ) -> Result<tonic::Response<super::PingResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/geyser.Geyser/Ping");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod geyser_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with GeyserServer.
    #[async_trait]
    pub trait Geyser: Send + Sync + 'static {
        /// Server streaming response type for the Subscribe method.
        type SubscribeStream: futures_core::Stream<Item = Result<super::SubscribeUpdate, tonic::Status>>
            + Send
            + 'static;
        async fn subscribe(
            &self,
            request: tonic::Request<tonic::Streaming<super::SubscribeRequest>>,
        ) -> Result<tonic::Response<Self::SubscribeStream>, tonic::Status>;
        async fn ping(
            &self,
            request: tonic::Request<super::PingRequest>,
        ) -> Result<tonic::Response<super::PingResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct GeyserServer<T: Geyser> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Geyser> GeyserServer<T> {
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GeyserServer<T>
    where
        T: Geyser,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/geyser.Geyser/Subscribe" => {
                    #[allow(non_camel_case_types)]
                    struct SubscribeSvc<T: Geyser>(pub Arc<T>);
                    impl<T: Geyser> tonic::server::StreamingService<super::SubscribeRequest> for SubscribeSvc<T> {
                        type Response = super::SubscribeUpdate;
                        type ResponseStream = T::SubscribeStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::SubscribeRequest>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).subscribe(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubscribeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/geyser.Geyser/Ping" => {
                    #[allow(non_camel_case_types)]
                    struct PingSvc<T: Geyser>(pub Arc<T>);
                    impl<T: Geyser> tonic::server::UnaryService<super::PingRequest> for PingSvc<T> {
                        type Response = super::PingResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PingRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).ping(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Geyser> Clone for GeyserServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Geyser> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Geyser> tonic::server::NamedService for GeyserServer<T> {
        const NAME: &'static str = "geyser.Geyser";
    }
}
