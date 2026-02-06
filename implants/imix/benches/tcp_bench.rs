use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use httptest::{matchers::*, responders::*, Expectation, Server};
use imix::portal::tcp::handle_tcp;
use pb::portal::{mote::Payload, Mote, TcpPayload};
use portal_stream::PayloadSequencer;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use std::time::Duration;

fn bench_tcp_download(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Start a mock HTTP server
    let server = Server::run();
    // 100MB payload
    let payload_size = 100 * 1024 * 1024;
    let data = vec![b'x'; payload_size];

    server.expect(
        Expectation::matching(request::method_path("GET", "/bigfile"))
        .respond_with(
            status_code(200)
            .body(data)
        )
    );

    let addr = server.addr();
    let dst_addr = addr.ip().to_string();
    let dst_port = addr.port() as u32;

    let mut group = c.benchmark_group("tcp_portal");
    group.throughput(Throughput::Bytes(payload_size as u64));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("download_100mb", |b| {
        b.to_async(&rt).iter(|| async {
            // Setup channels
            // rx for handle_tcp (C2 -> Socket). We close it immediately as we only send initial data.
            let (rx_tx, rx) = mpsc::channel::<Mote>(1);
            drop(rx_tx); // Close it immediately

            // out_tx for handle_tcp (Socket -> C2)
            let (out_tx, mut out_rx) = mpsc::channel::<Mote>(100);

            // Initial request
            let initial_request = format!("GET /bigfile HTTP/1.1\r\nHost: {}\r\n\r\n", addr);
            let first_mote = Mote {
                stream_id: "bench".into(),
                seq_id: 0,
                payload: Some(Payload::Tcp(TcpPayload {
                    dst_addr: dst_addr.clone(),
                    dst_port,
                    data: initial_request.into_bytes(),
                })),
            };

            let sequencer = PayloadSequencer::new("bench");

            // Spawn consumer
            let consumer = tokio::spawn(async move {
                let mut total_bytes = 0;
                while let Some(mote) = out_rx.recv().await {
                    if let Some(Payload::Tcp(tcp)) = mote.payload {
                         total_bytes += tcp.data.len();
                    }
                    if total_bytes >= payload_size {
                        break;
                    }
                }
            });

            // Run portal
            let _ = handle_tcp(first_mote, rx, out_tx, sequencer).await;

            // Wait for consumer
            let _ = consumer.await;
        })
    });
    group.finish();
}

criterion_group!(benches, bench_tcp_download);
criterion_main!(benches);
