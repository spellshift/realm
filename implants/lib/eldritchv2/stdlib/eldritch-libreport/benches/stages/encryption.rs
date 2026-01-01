use criterion::{BenchmarkId, Criterion};

use crate::fixtures;

/// Benchmark XChaCha20-Poly1305 encryption overhead
pub fn bench_encryption_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage2_encryption");

    // Test different chunk sizes
    let chunk_sizes = vec![
        1024,              // 1 KB
        256 * 1024,        // 256 KB
        1024 * 1024,       // 1 MB
        2 * 1024 * 1024,   // 2 MB
    ];

    for chunk_size in chunk_sizes {
        group.throughput(criterion::Throughput::Bytes(chunk_size as u64));

        // Benchmark encryption throughput
        group.bench_with_input(
            BenchmarkId::new("xchacha20_encrypt", fixtures::human_readable(chunk_size)),
            &chunk_size,
            |b, &size| {
                // Pre-generate plaintext data
                let plaintext = vec![0xCD; size];
                // Create a dummy request for encryption
                use pb::c2::ReportFileRequest;
                use pb::eldritch::{File, FileMetadata};

                b.iter(|| {
                    let req = ReportFileRequest {
                        task_id: 1,
                        chunk: Some(File {
                            metadata: None,
                            chunk: plaintext.clone(),
                        }),
                    };

                    // Encrypt using public API
                    let ciphertext = pb::xchacha::encode_with_chacha::<
                        ReportFileRequest,
                        pb::c2::ReportFileResponse,
                    >(req).unwrap();
                    std::hint::black_box(ciphertext);
                });
            },
        );

        // Benchmark encryption size overhead
        group.bench_with_input(
            BenchmarkId::new("encryption_overhead", fixtures::human_readable(chunk_size)),
            &chunk_size,
            |b, &size| {
                let plaintext = vec![0xCD; size];
                use pb::c2::ReportFileRequest;
                use pb::eldritch::File;

                b.iter(|| {
                    let req = ReportFileRequest {
                        task_id: 1,
                        chunk: Some(File {
                            metadata: None,
                            chunk: plaintext.clone(),
                        }),
                    };

                    let plaintext_encoded = prost::Message::encode_to_vec(&req);
                    let ciphertext = pb::xchacha::encode_with_chacha::<
                        ReportFileRequest,
                        pb::c2::ReportFileResponse,
                    >(req).unwrap();
                    let overhead = ciphertext.len() - plaintext_encoded.len();
                    std::hint::black_box(overhead); // Should be 72 bytes (32+24+16)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark the cost of X25519 key generation (per-message overhead)
pub fn bench_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage2_key_generation");

    // Measure key generation cost (happens once per encrypted message)
    group.bench_function("x25519_ephemeral_key", |b| {
        b.iter(|| {
            use rand_chacha::ChaCha20Rng;
            use rand_core::SeedableRng;
            use x25519_dalek::EphemeralSecret;

            let mut rng = ChaCha20Rng::from_entropy();
            let secret = EphemeralSecret::random_from_rng(&mut rng);
            std::hint::black_box(secret);
        });
    });

    group.finish();
}
