use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use eldritch_libreport::ReportLibrary;
use eldritch_libreport::std::StdReportLibrary;
use std::sync::Arc;
use std::time::Instant;

mod fixtures;
mod mocks;

use mocks::RealisticAgent;

/// Helper macro to reduce boilerplate
macro_rules! bench_network_case {
    ($group:expr, $name:expr, $file_size:expr, $agent_fn:expr) => {
        $group.throughput(Throughput::Bytes($file_size as u64));
        $group.bench_function($name, |b| {
            b.iter_batched(
                || {
                    let temp_file = fixtures::create_test_file($file_size);
                    let agent_arc = Arc::new($agent_fn());
                    let library = StdReportLibrary::new(agent_arc, 1);
                    (temp_file, library)
                },
                |(temp_file, library)| {
                    let start = Instant::now();
                    library
                        .file(temp_file.path().to_string_lossy().to_string())
                        .unwrap();
                    start.elapsed()
                },
                criterion::BatchSize::SmallInput,
            );
        });
    };
}

/// Benchmark end-to-end file reporting with different network conditions
fn bench_end_to_end_with_network_conditions(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // Test matrix: file size × network conditions
    bench_network_case!(group, "1KB/fast", 1024, RealisticAgent::fast_network);
    bench_network_case!(group, "1MB/typical", 1024 * 1024, RealisticAgent::typical_network);
    bench_network_case!(group, "1MB/slow", 1024 * 1024, RealisticAgent::slow_network);
    bench_network_case!(group, "5MB/typical", 5 * 1024 * 1024, RealisticAgent::typical_network);
    bench_network_case!(group, "5MB/slow", 5 * 1024 * 1024, RealisticAgent::slow_network);
    bench_network_case!(group, "10MB/fast", 10 * 1024 * 1024, RealisticAgent::fast_network);
    bench_network_case!(group, "10MB/typical", 10 * 1024 * 1024, RealisticAgent::typical_network);

    group.finish();
}

/// Benchmark latency breakdown
/// Measures time to first byte vs total time
fn bench_latency_breakdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_breakdown");

    let file_sizes = vec![
        1024,              // 1 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        group.throughput(Throughput::Bytes(file_size as u64));

        group.bench_with_input(
            BenchmarkId::new("total_time", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let temp_file = fixtures::create_test_file(size);
                        let agent = Arc::new(RealisticAgent::typical_network());
                        let library = StdReportLibrary::new(agent, 1);
                        (temp_file, library)
                    },
                    |(temp_file, library)| {
                        let start = Instant::now();
                        library
                            .file(temp_file.path().to_string_lossy().to_string())
                            .unwrap();
                        start.elapsed()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark network efficiency
/// Measures encrypted bytes vs raw bytes overhead
fn bench_network_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_efficiency");

    let file_sizes = vec![
        1024,              // 1 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
    ];

    for file_size in file_sizes {
        group.bench_with_input(
            BenchmarkId::new("overhead_ratio", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let temp_file = fixtures::create_test_file(size);
                        let agent = Arc::new(RealisticAgent::fast_network());
                        let library = StdReportLibrary::new(agent.clone(), 1);
                        (temp_file, library, agent)
                    },
                    |(temp_file, library, agent)| {
                        agent.reset_metrics();
                        library
                            .file(temp_file.path().to_string_lossy().to_string())
                            .unwrap();

                        let metrics = agent.metrics();
                        let overhead_ratio = if metrics.total_bytes > 0 {
                            (metrics.encrypted_bytes - metrics.total_bytes) as f64
                                / metrics.total_bytes as f64
                        } else {
                            0.0
                        };

                        (
                            metrics.total_bytes,
                            metrics.encrypted_bytes,
                            metrics.chunks,
                            overhead_ratio,
                        )
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark throughput for different file sizes
fn bench_throughput_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let file_sizes = vec![
        1024,              // 1 KB
        256 * 1024,        // 256 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        group.throughput(Throughput::Bytes(file_size as u64));

        group.bench_with_input(
            BenchmarkId::new("mb_per_sec", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let temp_file = fixtures::create_test_file(size);
                        let agent = Arc::new(RealisticAgent::fast_network());
                        let library = StdReportLibrary::new(agent, 1);
                        (temp_file, library)
                    },
                    |(temp_file, library)| {
                        let start = Instant::now();
                        library
                            .file(temp_file.path().to_string_lossy().to_string())
                            .unwrap();
                        let elapsed = start.elapsed();

                        // Calculate throughput in MB/s
                        let throughput_mbps = if elapsed.as_secs_f64() > 0.0 {
                            (size as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64()
                        } else {
                            0.0
                        };

                        throughput_mbps
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
    bench_end_to_end_with_network_conditions,
    bench_latency_breakdown,
    bench_network_efficiency,
    bench_throughput_by_size,
);
criterion_main!(benches);
