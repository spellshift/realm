use criterion::{criterion_group, criterion_main, Criterion};
use eldritch::runtime::messages::ReverseShellPTYMessage;
use eldritch::runtime::messages::AsyncDispatcher;
use transport::MockTransport;
use transport::Transport;
use pb::config::Config;
use pb::c2::{ReverseShellRequest, ReverseShellResponse, ReverseShellMessageKind};
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};

fn bench_reverse_shell(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup the environment and channels once
    let (mut output_rx, input_tx) = rt.block_on(async {
        let mut transport = MockTransport::default();

        let (tx_handoff, mut rx_handoff) = tokio::sync::mpsc::unbounded_channel();
        let tx_handoff = Arc::new(Mutex::new(Some(tx_handoff)));

        transport.expect_reverse_shell()
            .times(1)
            .returning(move |output_rx, input_tx| {
                if let Some(tx) = tx_handoff.lock().unwrap().take() {
                    let _ = tx.send((output_rx, input_tx));
                }
                Ok(())
            });

        // Use a simple command that echoes input
        let msg = ReverseShellPTYMessage::new(123, Some("cat".to_string()));

        tokio::spawn(async move {
            let mut t = transport;
            // Config default might not be public or derived, check pb
            // If Config doesn't implement Default, we can construct it if fields are pub
            // But Config::default() is usually available if derived.
            // If not, we can use `unsafe { std::mem::zeroed() }` or similar if really desperate (bad idea).
            // pb::config::Config fields are public.
            // Let's assume Default is implemented or we construct it.
            let cfg = Config {
               // ... fill if needed.
               // Check if Config implements Default
               ..Default::default()
            };
            let _ = msg.dispatch(&mut t, cfg).await;
        });

        let (output_rx, input_tx) = rx_handoff.recv().await.unwrap();
        (output_rx, input_tx)
    });

    // Drain initial Ping
    rt.block_on(async {
        if let Some(msg) = output_rx.recv().await {
            assert_eq!(msg.kind, ReverseShellMessageKind::Ping as i32);
        }
    });

    let output_rx = Arc::new(tokio::sync::Mutex::new(output_rx));
    let input_tx = input_tx.clone();

    c.bench_function("reverse_shell_echo_roundtrip", |b| {
        b.to_async(&rt).iter(|| async {
            let tx = input_tx.clone();
            let rx = output_rx.clone();

            let data = b"bench\n".to_vec();

            tx.send(ReverseShellResponse {
                kind: ReverseShellMessageKind::Data.into(),
                data: data.clone(),
            }).await.unwrap();

            // Read until we get "bench" back
            let mut received = Vec::new();

            loop {
                let mut guard = rx.lock().await;
                let msg = match guard.recv().await {
                    Some(m) => m,
                    None => break,
                };
                drop(guard);

                if msg.kind == ReverseShellMessageKind::Data as i32 {
                    received.extend_from_slice(&msg.data);
                    if String::from_utf8_lossy(&received).contains("bench") {
                        break;
                    }
                }
            }
        })
    });
}

criterion_group!(benches, bench_reverse_shell);
criterion_main!(benches);
