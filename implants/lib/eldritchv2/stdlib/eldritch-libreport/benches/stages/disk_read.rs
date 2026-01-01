use criterion::{BenchmarkId, Criterion};
use std::fs::File;
use std::io::Read;

use crate::fixtures;

/// Benchmark different disk read strategies
pub fn bench_disk_read_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("stage1_disk_read");

    let file_sizes = vec![
        1024,              // 1 KB
        1024 * 1024,       // 1 MB
        5 * 1024 * 1024,   // 5 MB
        10 * 1024 * 1024,  // 10 MB
    ];

    for file_size in file_sizes {
        group.throughput(criterion::Throughput::Bytes(file_size as u64));

        // Strategy 1: std::fs::read (current v2 - full file into memory)
        group.bench_with_input(
            BenchmarkId::new("full_read", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        let path = temp_file.path();
                        let content = std::fs::read(path).unwrap();
                        std::hint::black_box(content);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 2: Chunked read with 1KB chunks (old v1 approach)
        group.bench_with_input(
            BenchmarkId::new("chunked_1kb", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        let path = temp_file.path();
                        let mut file = File::open(path).unwrap();
                        let mut buffer = [0u8; 1024];
                        let mut total_read = 0;

                        while total_read < size {
                            let n = file.read(&mut buffer).unwrap();
                            if n == 0 {
                                break;
                            }
                            total_read += n;
                            std::hint::black_box(&buffer[..n]);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 3: Chunked read with 256KB chunks
        group.bench_with_input(
            BenchmarkId::new("chunked_256kb", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        let path = temp_file.path();
                        let mut file = File::open(path).unwrap();
                        let mut buffer = vec![0u8; 256 * 1024];
                        let mut total_read = 0;

                        while total_read < size {
                            let n = file.read(&mut buffer).unwrap();
                            if n == 0 {
                                break;
                            }
                            total_read += n;
                            std::hint::black_box(&buffer[..n]);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Strategy 4: Chunked read with 2MB chunks (new streaming approach)
        group.bench_with_input(
            BenchmarkId::new("chunked_2mb", fixtures::human_readable(file_size)),
            &file_size,
            |b, &size| {
                b.iter_batched(
                    || fixtures::create_test_file(size),
                    |temp_file| {
                        let path = temp_file.path();
                        let mut file = File::open(path).unwrap();
                        let mut buffer = vec![0u8; 2 * 1024 * 1024];
                        let mut total_read = 0;

                        while total_read < size {
                            let n = file.read(&mut buffer).unwrap();
                            if n == 0 {
                                break;
                            }
                            total_read += n;
                            std::hint::black_box(&buffer[..n]);
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}
