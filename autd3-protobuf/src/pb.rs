#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vector3 {
    #[prost(float, tag = "1")]
    pub x: f32,
    #[prost(float, tag = "2")]
    pub y: f32,
    #[prost(float, tag = "3")]
    pub z: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Quaternion {
    #[prost(float, tag = "1")]
    pub w: f32,
    #[prost(float, tag = "2")]
    pub x: f32,
    #[prost(float, tag = "3")]
    pub y: f32,
    #[prost(float, tag = "4")]
    pub z: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Geometry {
    #[prost(message, repeated, tag = "1")]
    pub devices: ::prost::alloc::vec::Vec<geometry::Autd3>,
}
/// Nested message and enum types in `Geometry`.
pub mod geometry {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Autd3 {
        #[prost(message, optional, tag = "1")]
        pub pos: ::core::option::Option<super::Vector3>,
        #[prost(message, optional, tag = "2")]
        pub rot: ::core::option::Option<super::Quaternion>,
        #[prost(float, tag = "3")]
        pub sound_speed: f32,
        #[prost(float, tag = "4")]
        pub attenuation: f32,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmitIntensity {
    #[prost(uint32, tag = "1")]
    pub value: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Phase {
    #[prost(uint32, tag = "1")]
    pub value: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SamplingConfiguration {
    #[prost(uint32, tag = "1")]
    pub freq_div: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoopBehavior {
    #[prost(uint32, tag = "1")]
    pub rep: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Segment {
    S0 = 0,
    S1 = 1,
}
impl Segment {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Segment::S0 => "S0",
            Segment::S1 => "S1",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "S0" => Some(Self::S0),
            "S1" => Some(Self::S1),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxRawData {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag = "2")]
    pub num_devices: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RxMessage {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GeometryResponse {}
/// Generated client implementations.
pub mod simulator_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct SimulatorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SimulatorClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SimulatorClient<T>
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
        ) -> SimulatorClient<InterceptedService<T, F>>
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
            SimulatorClient::new(InterceptedService::new(inner, interceptor))
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn config_geomety(
            &mut self,
            request: impl tonic::IntoRequest<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::GeometryResponse>,
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
                "/autd3.Simulator/ConfigGeomety",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("autd3.Simulator", "ConfigGeomety"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_geomety(
            &mut self,
            request: impl tonic::IntoRequest<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::GeometryResponse>,
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
                "/autd3.Simulator/UpdateGeomety",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("autd3.Simulator", "UpdateGeomety"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn send_data(
            &mut self,
            request: impl tonic::IntoRequest<super::TxRawData>,
        ) -> std::result::Result<tonic::Response<super::SendResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.Simulator/SendData");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.Simulator", "SendData"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn read_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadRequest>,
        ) -> std::result::Result<tonic::Response<super::RxMessage>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.Simulator/ReadData");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.Simulator", "ReadData"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn close(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseRequest>,
        ) -> std::result::Result<tonic::Response<super::CloseResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.Simulator/Close");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.Simulator", "Close"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod ecat_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct EcatClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EcatClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> EcatClient<T>
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
        ) -> EcatClient<InterceptedService<T, F>>
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
            EcatClient::new(InterceptedService::new(inner, interceptor))
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn send_data(
            &mut self,
            request: impl tonic::IntoRequest<super::TxRawData>,
        ) -> std::result::Result<tonic::Response<super::SendResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.ECAT/SendData");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.ECAT", "SendData"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn read_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadRequest>,
        ) -> std::result::Result<tonic::Response<super::RxMessage>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.ECAT/ReadData");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.ECAT", "ReadData"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn close(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseRequest>,
        ) -> std::result::Result<tonic::Response<super::CloseResponse>, tonic::Status> {
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
            let path = http::uri::PathAndQuery::from_static("/autd3.ECAT/Close");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.ECAT", "Close"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod simulator_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SimulatorServer.
    #[async_trait]
    pub trait Simulator: Send + Sync + 'static {
        async fn config_geomety(
            &self,
            request: tonic::Request<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::GeometryResponse>,
            tonic::Status,
        >;
        async fn update_geomety(
            &self,
            request: tonic::Request<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::GeometryResponse>,
            tonic::Status,
        >;
        async fn send_data(
            &self,
            request: tonic::Request<super::TxRawData>,
        ) -> std::result::Result<tonic::Response<super::SendResponse>, tonic::Status>;
        async fn read_data(
            &self,
            request: tonic::Request<super::ReadRequest>,
        ) -> std::result::Result<tonic::Response<super::RxMessage>, tonic::Status>;
        async fn close(
            &self,
            request: tonic::Request<super::CloseRequest>,
        ) -> std::result::Result<tonic::Response<super::CloseResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SimulatorServer<T: Simulator> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Simulator> SimulatorServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SimulatorServer<T>
    where
        T: Simulator,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/autd3.Simulator/ConfigGeomety" => {
                    #[allow(non_camel_case_types)]
                    struct ConfigGeometySvc<T: Simulator>(pub Arc<T>);
                    impl<T: Simulator> tonic::server::UnaryService<super::Geometry>
                    for ConfigGeometySvc<T> {
                        type Response = super::GeometryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Geometry>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Simulator>::config_geomety(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConfigGeometySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.Simulator/UpdateGeomety" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateGeometySvc<T: Simulator>(pub Arc<T>);
                    impl<T: Simulator> tonic::server::UnaryService<super::Geometry>
                    for UpdateGeometySvc<T> {
                        type Response = super::GeometryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Geometry>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Simulator>::update_geomety(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateGeometySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.Simulator/SendData" => {
                    #[allow(non_camel_case_types)]
                    struct SendDataSvc<T: Simulator>(pub Arc<T>);
                    impl<T: Simulator> tonic::server::UnaryService<super::TxRawData>
                    for SendDataSvc<T> {
                        type Response = super::SendResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxRawData>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Simulator>::send_data(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.Simulator/ReadData" => {
                    #[allow(non_camel_case_types)]
                    struct ReadDataSvc<T: Simulator>(pub Arc<T>);
                    impl<T: Simulator> tonic::server::UnaryService<super::ReadRequest>
                    for ReadDataSvc<T> {
                        type Response = super::RxMessage;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Simulator>::read_data(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.Simulator/Close" => {
                    #[allow(non_camel_case_types)]
                    struct CloseSvc<T: Simulator>(pub Arc<T>);
                    impl<T: Simulator> tonic::server::UnaryService<super::CloseRequest>
                    for CloseSvc<T> {
                        type Response = super::CloseResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloseRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Simulator>::close(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CloseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
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
    impl<T: Simulator> Clone for SimulatorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Simulator> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Simulator> tonic::server::NamedService for SimulatorServer<T> {
        const NAME: &'static str = "autd3.Simulator";
    }
}
/// Generated server implementations.
pub mod ecat_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with EcatServer.
    #[async_trait]
    pub trait Ecat: Send + Sync + 'static {
        async fn send_data(
            &self,
            request: tonic::Request<super::TxRawData>,
        ) -> std::result::Result<tonic::Response<super::SendResponse>, tonic::Status>;
        async fn read_data(
            &self,
            request: tonic::Request<super::ReadRequest>,
        ) -> std::result::Result<tonic::Response<super::RxMessage>, tonic::Status>;
        async fn close(
            &self,
            request: tonic::Request<super::CloseRequest>,
        ) -> std::result::Result<tonic::Response<super::CloseResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct EcatServer<T: Ecat> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Ecat> EcatServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EcatServer<T>
    where
        T: Ecat,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/autd3.ECAT/SendData" => {
                    #[allow(non_camel_case_types)]
                    struct SendDataSvc<T: Ecat>(pub Arc<T>);
                    impl<T: Ecat> tonic::server::UnaryService<super::TxRawData>
                    for SendDataSvc<T> {
                        type Response = super::SendResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TxRawData>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Ecat>::send_data(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.ECAT/ReadData" => {
                    #[allow(non_camel_case_types)]
                    struct ReadDataSvc<T: Ecat>(pub Arc<T>);
                    impl<T: Ecat> tonic::server::UnaryService<super::ReadRequest>
                    for ReadDataSvc<T> {
                        type Response = super::RxMessage;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ReadRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Ecat>::read_data(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReadDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.ECAT/Close" => {
                    #[allow(non_camel_case_types)]
                    struct CloseSvc<T: Ecat>(pub Arc<T>);
                    impl<T: Ecat> tonic::server::UnaryService<super::CloseRequest>
                    for CloseSvc<T> {
                        type Response = super::CloseResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloseRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Ecat>::close(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CloseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
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
    impl<T: Ecat> Clone for EcatServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Ecat> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Ecat> tonic::server::NamedService for EcatServer<T> {
        const NAME: &'static str = "autd3.ECAT";
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bessel {
    #[prost(message, optional, tag = "1")]
    pub intensity: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "2")]
    pub pos: ::core::option::Option<Vector3>,
    #[prost(message, optional, tag = "3")]
    pub dir: ::core::option::Option<Vector3>,
    #[prost(float, tag = "4")]
    pub theta: f32,
    #[prost(message, optional, tag = "5")]
    pub phase_offset: ::core::option::Option<Phase>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Focus {
    #[prost(message, optional, tag = "1")]
    pub intensity: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "2")]
    pub pos: ::core::option::Option<Vector3>,
    #[prost(message, optional, tag = "3")]
    pub phase_offset: ::core::option::Option<Phase>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Null {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Plane {
    #[prost(message, optional, tag = "1")]
    pub intensity: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "2")]
    pub dir: ::core::option::Option<Vector3>,
    #[prost(message, optional, tag = "3")]
    pub phase_offset: ::core::option::Option<Phase>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uniform {
    #[prost(message, optional, tag = "1")]
    pub intensity: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "2")]
    pub phase: ::core::option::Option<Phase>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Amplitude {
    #[prost(float, tag = "1")]
    pub value: f32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Holo {
    #[prost(message, optional, tag = "1")]
    pub pos: ::core::option::Option<Vector3>,
    #[prost(message, optional, tag = "2")]
    pub amp: ::core::option::Option<Amplitude>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DontCareConstraint {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NormalizeConstraint {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UniformConstraint {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<EmitIntensity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClampConstraint {
    #[prost(message, optional, tag = "1")]
    pub min: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "2")]
    pub max: ::core::option::Option<EmitIntensity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmissionConstraint {
    #[prost(oneof = "emission_constraint::Constraint", tags = "1, 2, 3, 4")]
    pub constraint: ::core::option::Option<emission_constraint::Constraint>,
}
/// Nested message and enum types in `EmissionConstraint`.
pub mod emission_constraint {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Constraint {
        #[prost(message, tag = "1")]
        DontCare(super::DontCareConstraint),
        #[prost(message, tag = "2")]
        Normalize(super::NormalizeConstraint),
        #[prost(message, tag = "3")]
        Uniform(super::UniformConstraint),
        #[prost(message, tag = "4")]
        Clamp(super::ClampConstraint),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sdp {
    #[prost(message, repeated, tag = "2")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(float, tag = "3")]
    pub alpha: f32,
    #[prost(float, tag = "4")]
    pub lambda: f32,
    #[prost(uint64, tag = "5")]
    pub repeat: u64,
    #[prost(message, optional, tag = "6")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Naive {
    #[prost(message, repeated, tag = "2")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(message, optional, tag = "3")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Gs {
    #[prost(message, repeated, tag = "2")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(uint64, tag = "3")]
    pub repeat: u64,
    #[prost(message, optional, tag = "4")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Gspat {
    #[prost(message, repeated, tag = "2")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(uint64, tag = "3")]
    pub repeat: u64,
    #[prost(message, optional, tag = "4")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lm {
    #[prost(message, repeated, tag = "2")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(float, tag = "3")]
    pub eps_1: f32,
    #[prost(float, tag = "4")]
    pub eps_2: f32,
    #[prost(float, tag = "5")]
    pub tau: f32,
    #[prost(uint64, tag = "6")]
    pub k_max: u64,
    #[prost(float, repeated, tag = "7")]
    pub initial: ::prost::alloc::vec::Vec<f32>,
    #[prost(message, optional, tag = "8")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Greedy {
    #[prost(message, repeated, tag = "1")]
    pub holo: ::prost::alloc::vec::Vec<Holo>,
    #[prost(uint32, tag = "2")]
    pub phase_div: u32,
    #[prost(message, optional, tag = "3")]
    pub constraint: ::core::option::Option<EmissionConstraint>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Gain {
    #[prost(enumeration = "Segment", tag = "1001")]
    pub segment: i32,
    #[prost(bool, tag = "1002")]
    pub update_segment: bool,
    #[prost(oneof = "gain::Gain", tags = "1, 2, 3, 4, 5, 100, 101, 102, 103, 104, 105")]
    pub gain: ::core::option::Option<gain::Gain>,
}
/// Nested message and enum types in `Gain`.
pub mod gain {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Gain {
        #[prost(message, tag = "1")]
        Bessel(super::Bessel),
        #[prost(message, tag = "2")]
        Focus(super::Focus),
        #[prost(message, tag = "3")]
        Null(super::Null),
        #[prost(message, tag = "4")]
        Plane(super::Plane),
        #[prost(message, tag = "5")]
        Uniform(super::Uniform),
        #[prost(message, tag = "100")]
        Sdp(super::Sdp),
        #[prost(message, tag = "101")]
        Naive(super::Naive),
        #[prost(message, tag = "102")]
        Gs(super::Gs),
        #[prost(message, tag = "103")]
        Gspat(super::Gspat),
        #[prost(message, tag = "104")]
        Lm(super::Lm),
        #[prost(message, tag = "105")]
        Greedy(super::Greedy),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Static {
    #[prost(message, optional, tag = "1")]
    pub intensity: ::core::option::Option<EmitIntensity>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sine {
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<SamplingConfiguration>,
    #[prost(float, tag = "2")]
    pub freq: f32,
    #[prost(message, optional, tag = "3")]
    pub intensity: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "4")]
    pub offset: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "5")]
    pub phase: ::core::option::Option<Phase>,
    #[prost(enumeration = "SamplingMode", tag = "6")]
    pub mode: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Square {
    #[prost(message, optional, tag = "1")]
    pub config: ::core::option::Option<SamplingConfiguration>,
    #[prost(float, tag = "2")]
    pub freq: f32,
    #[prost(message, optional, tag = "3")]
    pub low: ::core::option::Option<EmitIntensity>,
    #[prost(message, optional, tag = "4")]
    pub high: ::core::option::Option<EmitIntensity>,
    #[prost(float, tag = "5")]
    pub duty: f32,
    #[prost(enumeration = "SamplingMode", tag = "6")]
    pub mode: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Modulation {
    #[prost(enumeration = "Segment", tag = "1001")]
    pub segment: i32,
    #[prost(bool, tag = "1002")]
    pub update_segment: bool,
    #[prost(oneof = "modulation::Modulation", tags = "1, 2, 4")]
    pub modulation: ::core::option::Option<modulation::Modulation>,
}
/// Nested message and enum types in `Modulation`.
pub mod modulation {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Modulation {
        #[prost(message, tag = "1")]
        Static(super::Static),
        #[prost(message, tag = "2")]
        Sine(super::Sine),
        #[prost(message, tag = "4")]
        Square(super::Square),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SamplingMode {
    ExactFreq = 0,
    SizeOpt = 1,
}
impl SamplingMode {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SamplingMode::ExactFreq => "EXACT_FREQ",
            SamplingMode::SizeOpt => "SIZE_OPT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EXACT_FREQ" => Some(Self::ExactFreq),
            "SIZE_OPT" => Some(Self::SizeOpt),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Clear {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureSilencerFixedUpdateRate {
    #[prost(uint32, tag = "1")]
    pub value_intensity: u32,
    #[prost(uint32, tag = "2")]
    pub value_phase: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureSilencerFixedCompletionSteps {
    #[prost(uint32, tag = "1")]
    pub value_intensity: u32,
    #[prost(uint32, tag = "2")]
    pub value_phase: u32,
    #[prost(bool, tag = "3")]
    pub strict_mode: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureSilencer {
    #[prost(oneof = "configure_silencer::Config", tags = "1, 2")]
    pub config: ::core::option::Option<configure_silencer::Config>,
}
/// Nested message and enum types in `ConfigureSilencer`.
pub mod configure_silencer {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Config {
        #[prost(message, tag = "1")]
        FixedUpdateRate(super::ConfigureSilencerFixedUpdateRate),
        #[prost(message, tag = "2")]
        FixedCompletionSteps(super::ConfigureSilencerFixedCompletionSteps),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Synchronize {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureForceFan {
    #[prost(bool, repeated, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureReadsFpgaState {
    #[prost(bool, repeated, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<bool>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConfigureDebugOutputIdx {
    #[prost(int32, repeated, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GainStm {
    #[prost(uint32, tag = "1")]
    pub freq_div: u32,
    #[prost(message, optional, tag = "2")]
    pub loop_behavior: ::core::option::Option<LoopBehavior>,
    #[prost(enumeration = "Segment", tag = "3")]
    pub segment: i32,
    #[prost(bool, tag = "4")]
    pub update_segment: bool,
    #[prost(message, repeated, tag = "5")]
    pub gains: ::prost::alloc::vec::Vec<Gain>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FocusStm {
    #[prost(uint32, tag = "1")]
    pub freq_div: u32,
    #[prost(message, optional, tag = "2")]
    pub loop_behavior: ::core::option::Option<LoopBehavior>,
    #[prost(enumeration = "Segment", tag = "3")]
    pub segment: i32,
    #[prost(bool, tag = "4")]
    pub update_segment: bool,
    #[prost(message, repeated, tag = "5")]
    pub points: ::prost::alloc::vec::Vec<focus_stm::ControlPoint>,
}
/// Nested message and enum types in `FocusSTM`.
pub mod focus_stm {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ControlPoint {
        #[prost(message, optional, tag = "1")]
        pub intensity: ::core::option::Option<super::EmitIntensity>,
        #[prost(message, optional, tag = "2")]
        pub pos: ::core::option::Option<super::Vector3>,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeGainSegment {
    #[prost(enumeration = "Segment", tag = "1")]
    pub segment: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeFocusStmSegment {
    #[prost(enumeration = "Segment", tag = "1")]
    pub segment: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeGainStmSegment {
    #[prost(enumeration = "Segment", tag = "1")]
    pub segment: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeModulationSegment {
    #[prost(enumeration = "Segment", tag = "1")]
    pub segment: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DatagramLightweight {
    #[prost(
        oneof = "datagram_lightweight::Datagram",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15"
    )]
    pub datagram: ::core::option::Option<datagram_lightweight::Datagram>,
}
/// Nested message and enum types in `DatagramLightweight`.
pub mod datagram_lightweight {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Datagram {
        #[prost(message, tag = "1")]
        Silencer(super::ConfigureSilencer),
        #[prost(message, tag = "2")]
        Modulation(super::Modulation),
        #[prost(message, tag = "3")]
        Gain(super::Gain),
        #[prost(message, tag = "4")]
        Clear(super::Clear),
        #[prost(message, tag = "5")]
        Synchronize(super::Synchronize),
        #[prost(message, tag = "6")]
        ForceFan(super::ConfigureForceFan),
        #[prost(message, tag = "7")]
        Debug(super::ConfigureDebugOutputIdx),
        #[prost(message, tag = "8")]
        ReadsFpgaState(super::ConfigureReadsFpgaState),
        #[prost(message, tag = "10")]
        FocusStm(super::FocusStm),
        #[prost(message, tag = "11")]
        GainStm(super::GainStm),
        #[prost(message, tag = "12")]
        ChangeGainSegment(super::ChangeGainSegment),
        #[prost(message, tag = "13")]
        ChangeGainStmSegment(super::ChangeGainStmSegment),
        #[prost(message, tag = "14")]
        ChangeFocusStmSegment(super::ChangeFocusStmSegment),
        #[prost(message, tag = "15")]
        ChangeModulationSegment(super::ChangeModulationSegment),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendResponseLightweight {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(bool, tag = "2")]
    pub err: bool,
    #[prost(string, tag = "3")]
    pub msg: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FirmwareInfoRequestLightweight {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FirmwareInfoResponseLightweight {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub msg: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub firmware_info_list: ::prost::alloc::vec::Vec<
        firmware_info_response_lightweight::FirmwareInfo,
    >,
}
/// Nested message and enum types in `FirmwareInfoResponseLightweight`.
pub mod firmware_info_response_lightweight {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FirmwareInfo {
        #[prost(uint32, tag = "1")]
        pub fpga_major_version: u32,
        #[prost(uint32, tag = "2")]
        pub fpga_minor_version: u32,
        #[prost(uint32, tag = "3")]
        pub cpu_major_version: u32,
        #[prost(uint32, tag = "4")]
        pub cpu_minor_version: u32,
        #[prost(uint32, tag = "5")]
        pub fpga_function_bits: u32,
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CloseRequestLightweight {}
/// Generated client implementations.
pub mod ecat_light_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct EcatLightClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EcatLightClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> EcatLightClient<T>
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
        ) -> EcatLightClient<InterceptedService<T, F>>
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
            EcatLightClient::new(InterceptedService::new(inner, interceptor))
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn config_geomety(
            &mut self,
            request: impl tonic::IntoRequest<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
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
                "/autd3.ECATLight/ConfigGeomety",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("autd3.ECATLight", "ConfigGeomety"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn firmware_info(
            &mut self,
            request: impl tonic::IntoRequest<super::FirmwareInfoRequestLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::FirmwareInfoResponseLightweight>,
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
                "/autd3.ECATLight/FirmwareInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("autd3.ECATLight", "FirmwareInfo"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn send(
            &mut self,
            request: impl tonic::IntoRequest<super::DatagramLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
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
            let path = http::uri::PathAndQuery::from_static("/autd3.ECATLight/Send");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.ECATLight", "Send"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn close(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseRequestLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
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
            let path = http::uri::PathAndQuery::from_static("/autd3.ECATLight/Close");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("autd3.ECATLight", "Close"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod ecat_light_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with EcatLightServer.
    #[async_trait]
    pub trait EcatLight: Send + Sync + 'static {
        async fn config_geomety(
            &self,
            request: tonic::Request<super::Geometry>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
            tonic::Status,
        >;
        async fn firmware_info(
            &self,
            request: tonic::Request<super::FirmwareInfoRequestLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::FirmwareInfoResponseLightweight>,
            tonic::Status,
        >;
        async fn send(
            &self,
            request: tonic::Request<super::DatagramLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
            tonic::Status,
        >;
        async fn close(
            &self,
            request: tonic::Request<super::CloseRequestLightweight>,
        ) -> std::result::Result<
            tonic::Response<super::SendResponseLightweight>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct EcatLightServer<T: EcatLight> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: EcatLight> EcatLightServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EcatLightServer<T>
    where
        T: EcatLight,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/autd3.ECATLight/ConfigGeomety" => {
                    #[allow(non_camel_case_types)]
                    struct ConfigGeometySvc<T: EcatLight>(pub Arc<T>);
                    impl<T: EcatLight> tonic::server::UnaryService<super::Geometry>
                    for ConfigGeometySvc<T> {
                        type Response = super::SendResponseLightweight;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Geometry>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EcatLight>::config_geomety(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConfigGeometySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.ECATLight/FirmwareInfo" => {
                    #[allow(non_camel_case_types)]
                    struct FirmwareInfoSvc<T: EcatLight>(pub Arc<T>);
                    impl<
                        T: EcatLight,
                    > tonic::server::UnaryService<super::FirmwareInfoRequestLightweight>
                    for FirmwareInfoSvc<T> {
                        type Response = super::FirmwareInfoResponseLightweight;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::FirmwareInfoRequestLightweight,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EcatLight>::firmware_info(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FirmwareInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.ECATLight/Send" => {
                    #[allow(non_camel_case_types)]
                    struct SendSvc<T: EcatLight>(pub Arc<T>);
                    impl<
                        T: EcatLight,
                    > tonic::server::UnaryService<super::DatagramLightweight>
                    for SendSvc<T> {
                        type Response = super::SendResponseLightweight;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DatagramLightweight>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EcatLight>::send(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/autd3.ECATLight/Close" => {
                    #[allow(non_camel_case_types)]
                    struct CloseSvc<T: EcatLight>(pub Arc<T>);
                    impl<
                        T: EcatLight,
                    > tonic::server::UnaryService<super::CloseRequestLightweight>
                    for CloseSvc<T> {
                        type Response = super::SendResponseLightweight;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloseRequestLightweight>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EcatLight>::close(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CloseSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
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
    impl<T: EcatLight> Clone for EcatLightServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: EcatLight> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: EcatLight> tonic::server::NamedService for EcatLightServer<T> {
        const NAME: &'static str = "autd3.ECATLight";
    }
}
