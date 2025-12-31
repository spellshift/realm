use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use transport::{ActiveTransport, Transport};
use pb::c2::{
    ClaimTasksRequest, ClaimTasksResponse, Beacon,
    ReportTaskOutputRequest, ReportTaskOutputResponse, TaskOutput
};
use pb::c2::c2_server::{C2, C2Server};
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use tokio::runtime::Runtime;

// Mock C2 Service Implementation
#[derive(Default)]
pub struct MockC2;

#[tonic::async_trait]
impl C2 for MockC2 {
    async fn claim_tasks(
        &self,
        _request: Request<ClaimTasksRequest>,
    ) -> Result<Response<ClaimTasksResponse>, Status> {
        Ok(Response::new(ClaimTasksResponse { tasks: vec![] }))
    }

    type FetchAssetStream = tokio_stream::wrappers::ReceiverStream<Result<pb::c2::FetchAssetResponse, Status>>;

    async fn fetch_asset(
        &self,
        _request: Request<pb::c2::FetchAssetRequest>,
    ) -> Result<Response<Self::FetchAssetStream>, Status> {
        Err(Status::unimplemented("not benchmarked"))
    }

    async fn report_credential(
        &self,
        _request: Request<pb::c2::ReportCredentialRequest>,
    ) -> Result<Response<pb::c2::ReportCredentialResponse>, Status> {
        Ok(Response::new(pb::c2::ReportCredentialResponse {}))
    }

    async fn report_file(
        &self,
        _request: Request<tonic::Streaming<pb::c2::ReportFileRequest>>,
    ) -> Result<Response<pb::c2::ReportFileResponse>, Status> {
        Ok(Response::new(pb::c2::ReportFileResponse {}))
    }

    async fn report_process_list(
        &self,
        _request: Request<pb::c2::ReportProcessListRequest>,
    ) -> Result<Response<pb::c2::ReportProcessListResponse>, Status> {
        Ok(Response::new(pb::c2::ReportProcessListResponse {}))
    }

    async fn report_task_output(
        &self,
        _request: Request<ReportTaskOutputRequest>,
    ) -> Result<Response<ReportTaskOutputResponse>, Status> {
        Ok(Response::new(ReportTaskOutputResponse {}))
    }

    type ReverseShellStream = tokio_stream::wrappers::ReceiverStream<Result<pb::c2::ReverseShellResponse, Status>>;

    async fn reverse_shell(
        &self,
        _request: Request<tonic::Streaming<pb::c2::ReverseShellRequest>>,
    ) -> Result<Response<Self::ReverseShellStream>, Status> {
        Err(Status::unimplemented("not benchmarked"))
    }

    type CreatePortalStream = tokio_stream::wrappers::ReceiverStream<Result<pb::c2::CreatePortalResponse, Status>>;

    async fn create_portal(
        &self,
        _request: Request<tonic::Streaming<pb::c2::CreatePortalRequest>>,
    ) -> Result<Response<Self::CreatePortalStream>, Status> {
        Err(Status::unimplemented("not benchmarked"))
    }
}

fn benchmark_grpc_transport(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter(); // Enter runtime context for IO objects

    // Start Mock Server on random port
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = std::net::TcpListener::bind(addr).unwrap();
    let local_addr = listener.local_addr().unwrap();

    listener.set_nonblocking(true).unwrap();
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(tokio::net::TcpListener::from_std(listener).unwrap());

    // Spawn server
    rt.spawn(async move {
        Server::builder()
            .add_service(C2Server::new(MockC2::default()))
            .serve_with_incoming(incoming)
            .await
            .unwrap();
    });

    let callback_uri = format!("http://{}", local_addr);

    // 1. Benchmark small control message (ClaimTasks)
    let mut group = c.benchmark_group("grpc_transport_throughput");
    group.throughput(Throughput::Elements(1));

    group.bench_function("claim_tasks_rps", |b| {
        b.to_async(&rt).iter(|| async {
            let mut transport = ActiveTransport::new(callback_uri.clone(), None).unwrap();

            let req = ClaimTasksRequest {
                beacon: Some(Beacon {
                    identifier: "bench".to_string(),
                    ..Default::default()
                }),
            };

            transport.claim_tasks(req).await.unwrap();
        })
    });

    // 2. Benchmark large payload upload (ReportTaskOutput)
    let payload_size = 1024 * 1024;
    let large_payload = "a".repeat(payload_size);
    group.throughput(Throughput::Bytes(payload_size as u64));

    group.bench_function("report_output_1MB", |b| {
        b.to_async(&rt).iter(|| async {
            let mut transport = ActiveTransport::new(callback_uri.clone(), None).unwrap();

            let req = ReportTaskOutputRequest {
                output: Some(TaskOutput {
                    id: 1,
                    output: large_payload.clone(),
                    ..Default::default()
                })
            };

            transport.report_task_output(req).await.unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_grpc_transport);
criterion_main!(benches);
