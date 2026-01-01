use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use prost::Message;

mod fixtures;
mod strategies;

/// Benchmark comparing different file reporting strategies
fn bench_comparative_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparative");

    // Test file sizes: 1KB, 1MB, 5MB, 10MB per user requirements
    // Also include 256KB and 2MB to find crossover points
    let file_sizes = vec![
        1024,              // 1 KB
        256 * 1024,        // 256 KB
        1024 * 1024,       // 1 MB
        2 * 1024 * 1024,   // 2 MB (chunk boundary)
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        group.throughput(Throughput::Bytes(file_size as u64));

        // Strategy 1: V2 Current (full file read, single chunk)
        group.bench_with_input(
            BenchmarkId::new("v2_current", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        strategies::v2_current::report_file(temp_file.path()).unwrap()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 2: V1 Legacy (1KB streaming)
        group.bench_with_input(
            BenchmarkId::new("v1_legacy_1kb", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        strategies::v1_legacy::report_file(temp_file.path()).unwrap()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 3: Streaming 2MB
        group.bench_with_input(
            BenchmarkId::new("streaming_2mb", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        strategies::streaming_2mb::report_file(temp_file.path()).unwrap()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 4: Adaptive chunking
        group.bench_with_input(
            BenchmarkId::new("adaptive", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        strategies::adaptive::report_file(temp_file.path()).unwrap()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark memory overhead comparison
/// Measures the memory cost of different strategies
fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    // Focus on larger files where memory differences are significant
    let file_sizes = vec![
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        // V2: Entire file in memory
        group.bench_with_input(
            BenchmarkId::new("v2_memory", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        // V2 allocates entire file + Vec overhead
                        let requests = strategies::v2_current::report_file(temp_file.path()).unwrap();
                        // Return total bytes allocated
                        requests.iter().map(|r| {
                            r.chunk.as_ref().map(|c| c.chunk.len()).unwrap_or(0)
                        }).sum::<usize>()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Streaming: Constant 2MB buffer regardless of file size
        group.bench_with_input(
            BenchmarkId::new("streaming_memory", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        // Streaming allocates chunks incrementally
                        let requests = strategies::streaming_2mb::report_file(temp_file.path()).unwrap();
                        // Return total bytes allocated (all chunks combined)
                        requests.iter().map(|r| {
                            r.chunk.as_ref().map(|c| c.chunk.len()).unwrap_or(0)
                        }).sum::<usize>()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark encryption overhead per strategy
/// V2: 1 large chunk → 72 bytes overhead total
/// Streaming: multiple chunks → 72 bytes × num_chunks overhead
fn bench_encryption_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_overhead");

    let file_sizes = vec![
        1024,              // 1 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
    ];

    for file_size in file_sizes {
        // V2: Single encryption operation
        group.bench_with_input(
            BenchmarkId::new("v2_encryption", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                let temp_file = fixtures::create_test_file(size);
                let requests = strategies::v2_current::report_file(temp_file.path()).unwrap();

                b.iter(|| {
                    let mut total_encrypted = 0;
                    let mut total_plaintext = 0;

                    for req in &requests {
                        let plaintext = req.encode_to_vec();
                        let ciphertext = pb::xchacha::encode_with_chacha::<
                            pb::c2::ReportFileRequest,
                            pb::c2::ReportFileResponse,
                        >(req.clone()).unwrap();
                        total_plaintext += plaintext.len();
                        total_encrypted += ciphertext.len();
                    }

                    (total_encrypted, total_plaintext, total_encrypted - total_plaintext)
                });
            },
        );

        // Streaming: Multiple encryption operations
        group.bench_with_input(
            BenchmarkId::new("streaming_encryption", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                let temp_file = fixtures::create_test_file(size);
                let requests = strategies::streaming_2mb::report_file(temp_file.path()).unwrap();

                b.iter(|| {
                    let mut total_encrypted = 0;
                    let mut total_plaintext = 0;

                    for req in &requests {
                        let plaintext = req.encode_to_vec();
                        let ciphertext = pb::xchacha::encode_with_chacha::<
                            pb::c2::ReportFileRequest,
                            pb::c2::ReportFileResponse,
                        >(req.clone()).unwrap();
                        total_plaintext += plaintext.len();
                        total_encrypted += ciphertext.len();
                    }

                    (total_encrypted, total_plaintext, total_encrypted - total_plaintext)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark chunk count analysis
/// Shows how many chunks each strategy produces
fn bench_chunk_counts(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_counts");

    let file_sizes = vec![
        1024,              // 1 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        group.bench_with_input(
            BenchmarkId::new("count_chunks", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        let v2 = strategies::v2_current::report_file(temp_file.path()).unwrap();
                        let v1 = strategies::v1_legacy::report_file(temp_file.path()).unwrap();
                        let streaming = strategies::streaming_2mb::report_file(temp_file.path()).unwrap();
                        let adaptive = strategies::adaptive::report_file(temp_file.path()).unwrap();

                        (v2.len(), v1.len(), streaming.len(), adaptive.len())
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_comparative_strategies,
    bench_memory_overhead,
    bench_encryption_overhead,
    bench_chunk_counts
);
criterion_main!(benches);
