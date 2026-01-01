use criterion::{criterion_group, criterion_main};

mod fixtures;
mod stages;

// Import all stage benchmarks
use stages::disk_read::*;
use stages::encryption::*;
use stages::serialization::*;

criterion_group!(
    benches,
    // Stage 1: Disk I/O
    bench_disk_read_strategies,
    // Stage 2: Encryption
    bench_encryption_overhead,
    bench_key_generation,
    // Stage 3: Serialization
    bench_protobuf_serialization,
    bench_metadata_overhead,
);
criterion_main!(benches);
