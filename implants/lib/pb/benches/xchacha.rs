//! Benchmark tests for XChaCha20-Poly1305 encryption/decryption operations.
//!
//! This benchmark suite evaluates the performance of `encode_with_chacha` (encryption)
//! and `decode_with_chacha` (decryption) functions with various payload sizes.
//!
//! # Running the benchmarks
//!
//! To run all benchmarks:
//! ```bash
//! cargo bench --bench xchacha
//! ```
//!
//! To run a specific benchmark group:
//! ```bash
//! cargo bench --bench xchacha encrypt_impl
//! cargo bench --bench xchacha decrypt_impl
//! cargo bench --bench xchacha roundtrip
//! ```
//!
//! To run with a specific payload size:
//! ```bash
//! cargo bench --bench xchacha -- /1024
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pb::c2::Agent;
use pb::xchacha::{decode_with_chacha, encode_with_chacha};

fn bench_encrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_impl");

    // Benchmark with different payload sizes
    for size in [
        64,      // Small message
        1024,    // 1 KB
        10240,   // 10 KB
        102400,  // 100 KB
        1048576, // 1 MB
    ] {
        // Create a message with the specified size
        let msg = Agent {
            identifier: "a".repeat(size),
        };

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &msg, |b, msg| {
            b.iter(|| encode_with_chacha::<Agent, Agent>(black_box(msg.clone())));
        });
    }

    group.finish();
}

fn bench_decrypt(c: &mut Criterion) {
    let mut group = c.benchmark_group("decrypt_impl");

    // Benchmark with different payload sizes
    for size in [
        64,      // Small message
        1024,    // 1 KB
        10240,   // 10 KB
        102400,  // 100 KB
        1048576, // 1 MB
    ] {
        // Create a message with the specified size
        let msg = Agent {
            identifier: "a".repeat(size),
        };

        // First encrypt to get valid ciphertext for decryption
        let encrypted = encode_with_chacha::<Agent, Agent>(msg.clone()).expect("Encryption failed");

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &encrypted,
            |b, encrypted| {
                b.iter(|| decode_with_chacha::<Agent, Agent>(black_box(encrypted.as_slice())));
            },
        );
    }

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("encrypt_decrypt_roundtrip");

    // Benchmark with different payload sizes
    for size in [
        102400,   // 100 KB
        1048576,  // 1 MB
        10485760, // 10 MB
    ] {
        // Create a message with the specified size
        let msg = Agent {
            identifier: "a".repeat(size),
        };

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &msg, |b, msg| {
            b.iter(|| {
                let encrypted = encode_with_chacha::<Agent, Agent>(black_box(msg.clone()))
                    .expect("Encryption failed");
                decode_with_chacha::<Agent, Agent>(black_box(encrypted.as_slice()))
                    .expect("Decryption failed")
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_encrypt, bench_decrypt, bench_roundtrip);
criterion_main!(benches);
