#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfaceStats {
    #[prost(string, tag = "1")]
    pub hostname: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "2")]
    pub port_stats: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        PortStats,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PortStats {
    #[prost(map = "string, message", tag = "1")]
    pub data: ::std::collections::HashMap<::prost::alloc::string::String, Data>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    #[prost(message, optional, tag = "3")]
    pub per_sec: ::core::option::Option<PerSec>,
    #[prost(message, optional, tag = "4")]
    pub elapsed: ::core::option::Option<Elapsed>,
    #[prost(oneof = "data::Data", tags = "1, 2")]
    pub data: ::core::option::Option<data::Data>,
}
/// Nested message and enum types in `Data`.
pub mod data {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "1")]
        Rxe(super::Rxe),
        #[prost(message, tag = "2")]
        Mlx(super::Mlx),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Elapsed {
    #[prost(uint64, tag = "1")]
    pub low: u64,
    #[prost(uint64, tag = "2")]
    pub high: u64,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PerSec {
    #[prost(double, tag = "3")]
    pub bytes_xmit_per_sec: f64,
    #[prost(double, tag = "4")]
    pub packets_xmit_per_sec: f64,
    #[prost(double, tag = "5")]
    pub bytes_rcv_per_sec: f64,
    #[prost(double, tag = "6")]
    pub packets_rcv_per_sec: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Rxe {
    #[prost(message, optional, tag = "1")]
    pub rxe_counter: ::core::option::Option<RxeCounter>,
    #[prost(message, optional, tag = "2")]
    pub rxe_hw_counter: ::core::option::Option<RxeHwCounter>,
}
#[path_resolver::derive_path::derive_path(
    path = "/sys/class/infiniband/{{ interface }}/ports/{{ port }}/hw_counters"
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RxeHwCounter {
    #[prost(uint64, tag = "1")]
    pub duplicate_request: u64,
    #[prost(uint64, tag = "2")]
    pub sent_pkts: u64,
    #[prost(uint64, tag = "3")]
    pub send_rnr_err: u64,
    #[prost(uint64, tag = "4")]
    pub send_err: u64,
    #[prost(uint64, tag = "5")]
    pub retry_rnr_exceeded_err: u64,
    #[prost(uint64, tag = "6")]
    pub retry_exceeded_err: u64,
    #[prost(uint64, tag = "7")]
    pub rdma_sends: u64,
    #[prost(uint64, tag = "8")]
    pub rdma_recvs: u64,
    #[prost(uint64, tag = "9")]
    pub rcvd_seq_err: u64,
    #[prost(uint64, tag = "10")]
    pub rcvd_rnr_err: u64,
    #[prost(uint64, tag = "11")]
    pub rcvd_pkts: u64,
    #[prost(uint64, tag = "12")]
    pub out_of_seq_request: u64,
    #[prost(uint64, tag = "13")]
    pub link_downed: u64,
    #[prost(uint64, tag = "14")]
    pub lifespan: u64,
    #[prost(uint64, tag = "15")]
    pub completer_retry_err: u64,
    #[prost(uint64, tag = "16")]
    pub ack_deferred: u64,
    #[prost(uint64, tag = "17")]
    pub port_xmit_data: u64,
}
#[path_resolver::derive_path::derive_path(
    path = "/sys/class/net/{{ linux_interface }}/statistics"
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RxeCounter {
    #[prost(uint64, tag = "1")]
    pub rx_bytes: u64,
    #[prost(uint64, tag = "2")]
    pub tx_bytes: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mlx {
    #[prost(message, optional, tag = "1")]
    pub mlx_counter: ::core::option::Option<MlxCounter>,
    #[prost(message, optional, tag = "2")]
    pub mlx_hw_counter: ::core::option::Option<MlxHwCounter>,
}
#[path_resolver::derive_path::derive_path(
    path = "/sys/class/infiniband/{{ interface }}/ports/{{ port }}/counters"
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MlxCounter {
    #[prost(uint64, tag = "1")]
    pub vl15_dropped: u64,
    #[prost(uint64, tag = "2")]
    pub excessive_buffer_overrun_errors: u64,
    #[prost(uint64, tag = "3")]
    pub link_downed: u64,
    #[prost(uint64, tag = "4")]
    pub link_error_recovery: u64,
    #[prost(uint64, tag = "5")]
    pub local_link_integrity_errors: u64,
    #[prost(uint64, tag = "6")]
    pub multicast_rcv_packets: u64,
    #[prost(uint64, tag = "7")]
    pub multicast_xmit_packets: u64,
    #[prost(uint64, tag = "8")]
    pub port_rcv_constraint_errors: u64,
    #[prost(uint64, tag = "9")]
    pub port_rcv_data: u64,
    #[prost(uint64, tag = "10")]
    pub port_rcv_errors: u64,
    #[prost(uint64, tag = "11")]
    pub port_rcv_packets: u64,
    #[prost(uint64, tag = "12")]
    pub port_rcv_remote_physical_errors: u64,
    #[prost(uint64, tag = "13")]
    pub port_rcv_switch_relay_errors: u64,
    #[prost(uint64, tag = "14")]
    pub port_xmit_constraint_errors: u64,
    #[prost(uint64, tag = "15")]
    pub port_xmit_data: u64,
    #[prost(uint64, tag = "16")]
    pub port_xmit_discards: u64,
    #[prost(uint64, tag = "17")]
    pub port_xmit_packets: u64,
    #[prost(uint64, tag = "18")]
    pub port_xmit_wait: u64,
    #[prost(uint64, tag = "19")]
    pub symbol_error: u64,
    #[prost(uint64, tag = "20")]
    pub unicast_rcv_packets: u64,
    #[prost(uint64, tag = "21")]
    pub unicast_xmit_packets: u64,
}
#[path_resolver::derive_path::derive_path(
    path = "/sys/class/infiniband/{{ interface }}/ports/{{ port }}/hw_counters"
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MlxHwCounter {
    #[prost(uint64, tag = "1")]
    pub duplicate_request: u64,
    #[prost(uint64, tag = "2")]
    pub implied_nak_seq_err: u64,
    #[prost(uint64, tag = "3")]
    pub lifespan: u64,
    #[prost(uint64, tag = "4")]
    pub local_ack_timeout_err: u64,
    #[prost(uint64, tag = "5")]
    pub np_cnp_sent: u64,
    #[prost(uint64, tag = "6")]
    pub np_ecn_marked_roce_packets: u64,
    #[prost(uint64, tag = "7")]
    pub out_of_buffer: u64,
    #[prost(uint64, tag = "8")]
    pub out_of_sequence: u64,
    #[prost(uint64, tag = "9")]
    pub packet_seq_err: u64,
    #[prost(uint64, tag = "10")]
    pub req_cqe_error: u64,
    #[prost(uint64, tag = "11")]
    pub req_cqe_flush_error: u64,
    #[prost(uint64, tag = "12")]
    pub req_remote_access_errors: u64,
    #[prost(uint64, tag = "13")]
    pub req_remote_invalid_request: u64,
    #[prost(uint64, tag = "14")]
    pub resp_cqe_error: u64,
    #[prost(uint64, tag = "15")]
    pub resp_cqe_flush_error: u64,
    #[prost(uint64, tag = "16")]
    pub resp_local_length_error: u64,
    #[prost(uint64, tag = "17")]
    pub resp_remote_access_errors: u64,
    #[prost(uint64, tag = "18")]
    pub rnr_nak_retry_err: u64,
    #[prost(uint64, tag = "19")]
    pub roce_adp_retrans: u64,
    #[prost(uint64, tag = "20")]
    pub roce_adp_retrans_to: u64,
    #[prost(uint64, tag = "21")]
    pub roce_slow_restart: u64,
    #[prost(uint64, tag = "22")]
    pub roce_slow_restart_cnps: u64,
    #[prost(uint64, tag = "23")]
    pub roce_slow_restart_trans: u64,
    #[prost(uint64, tag = "24")]
    pub rp_cnp_handled: u64,
    #[prost(uint64, tag = "25")]
    pub rp_cnp_ignored: u64,
    #[prost(uint64, tag = "26")]
    pub rx_atomic_requests: u64,
    #[prost(uint64, tag = "27")]
    pub rx_icrc_encapsulated: u64,
    #[prost(uint64, tag = "28")]
    pub rx_read_requests: u64,
    #[prost(uint64, tag = "29")]
    pub rx_write_requests: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReceiveReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Report {
    #[prost(string, tag = "1")]
    pub hostname: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub test_info: ::core::option::Option<TestInfo>,
    #[prost(message, optional, tag = "5")]
    pub bw_results: ::core::option::Option<BwResults>,
}
#[derive(serde::Deserialize, serde::Serialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BwResults {
    #[prost(uint32, tag = "1")]
    pub msg_size: u32,
    #[prost(uint32, tag = "2")]
    pub n_iterations: u32,
    #[prost(double, tag = "3")]
    pub bw_peak: f64,
    #[prost(double, tag = "4")]
    pub bw_average: f64,
    #[prost(double, tag = "5")]
    pub msg_rate: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestInfo {
    #[prost(string, tag = "1")]
    pub test: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub dual_port: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub device: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub number_of_qps: u32,
    #[prost(string, tag = "5")]
    pub transport_type: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub connection_type: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub using_srq: ::prost::alloc::string::String,
    #[prost(string, tag = "8")]
    pub pci_relax_order: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub ibv_wr_api: ::prost::alloc::string::String,
    #[prost(uint32, optional, tag = "10")]
    pub tx_depth: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag = "11")]
    pub rx_depth: ::core::option::Option<u32>,
    #[prost(uint32, tag = "12")]
    pub cq_moderation: u32,
    #[prost(uint32, tag = "13")]
    pub mtu: u32,
    #[prost(string, tag = "14")]
    pub link_type: ::prost::alloc::string::String,
    #[prost(uint32, tag = "15")]
    pub gid_index: u32,
    #[prost(uint32, tag = "16")]
    pub max_inline_data: u32,
    #[prost(string, tag = "17")]
    pub rdma_cm_qps: ::prost::alloc::string::String,
    #[prost(string, tag = "18")]
    pub data_ex_method: ::prost::alloc::string::String,
}
/// Generated client implementations.
pub mod monitor_server_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct MonitorServerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MonitorServerClient<tonic::transport::Channel> {
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
    impl<T> MonitorServerClient<T>
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
        ) -> MonitorServerClient<InterceptedService<T, F>>
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
            MonitorServerClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn send_stats(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::InterfaceStats>,
        ) -> std::result::Result<tonic::Response<super::ReceiveReply>, tonic::Status> {
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
                "/monitor.MonitorServer/SendStats",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("monitor.MonitorServer", "SendStats"));
            self.inner.client_streaming(req, path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod report_server_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ReportServerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ReportServerClient<tonic::transport::Channel> {
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
    impl<T> ReportServerClient<T>
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
        ) -> ReportServerClient<InterceptedService<T, F>>
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
            ReportServerClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn send_report(
            &mut self,
            request: impl tonic::IntoRequest<super::Report>,
        ) -> std::result::Result<tonic::Response<super::ReceiveReply>, tonic::Status> {
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
                "/monitor.ReportServer/SendReport",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("monitor.ReportServer", "SendReport"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod monitor_server_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MonitorServerServer.
    #[async_trait]
    pub trait MonitorServer: Send + Sync + 'static {
        async fn send_stats(
            &self,
            request: tonic::Request<tonic::Streaming<super::InterfaceStats>>,
        ) -> std::result::Result<tonic::Response<super::ReceiveReply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct MonitorServerServer<T: MonitorServer> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: MonitorServer> MonitorServerServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MonitorServerServer<T>
    where
        T: MonitorServer,
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
                "/monitor.MonitorServer/SendStats" => {
                    #[allow(non_camel_case_types)]
                    struct SendStatsSvc<T: MonitorServer>(pub Arc<T>);
                    impl<
                        T: MonitorServer,
                    > tonic::server::ClientStreamingService<super::InterfaceStats>
                    for SendStatsSvc<T> {
                        type Response = super::ReceiveReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<super::InterfaceStats>,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MonitorServer>::send_stats(&inner, request).await
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
                        let method = SendStatsSvc(inner);
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
                        let res = grpc.client_streaming(method, req).await;
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
    impl<T: MonitorServer> Clone for MonitorServerServer<T> {
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
    impl<T: MonitorServer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: MonitorServer> tonic::server::NamedService for MonitorServerServer<T> {
        const NAME: &'static str = "monitor.MonitorServer";
    }
}
/// Generated server implementations.
pub mod report_server_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ReportServerServer.
    #[async_trait]
    pub trait ReportServer: Send + Sync + 'static {
        async fn send_report(
            &self,
            request: tonic::Request<super::Report>,
        ) -> std::result::Result<tonic::Response<super::ReceiveReply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ReportServerServer<T: ReportServer> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ReportServer> ReportServerServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ReportServerServer<T>
    where
        T: ReportServer,
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
                "/monitor.ReportServer/SendReport" => {
                    #[allow(non_camel_case_types)]
                    struct SendReportSvc<T: ReportServer>(pub Arc<T>);
                    impl<T: ReportServer> tonic::server::UnaryService<super::Report>
                    for SendReportSvc<T> {
                        type Response = super::ReceiveReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Report>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ReportServer>::send_report(&inner, request).await
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
                        let method = SendReportSvc(inner);
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
    impl<T: ReportServer> Clone for ReportServerServer<T> {
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
    impl<T: ReportServer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ReportServer> tonic::server::NamedService for ReportServerServer<T> {
        const NAME: &'static str = "monitor.ReportServer";
    }
}
