use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use pb::xchacha;
use rand::Rng;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BenchmarkPayload {
    #[prost(bytes = "vec", tag = "1")]
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BenchmarkResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub data: Vec<u8>,
}

fn benchmark_xchacha(c: &mut Criterion) {
    let mut group = c.benchmark_group("xchacha_throughput");

    // Sizes: 1KB, 64KB, 256KB, 1MB, 5MB
    let sizes_kb = [1, 64, 256, 1024, 5120];

    for size_kb in sizes_kb.iter() {
        let size_bytes = size_kb * 1024;
        group.throughput(Throughput::Bytes(size_bytes as u64));

        let mut rng = rand::thread_rng();
        let data: Vec<u8> = (0..size_bytes).map(|_| rng.gen()).collect();
        let payload = BenchmarkPayload { data: data.clone() };

        group.bench_function(format!("encrypt_{}KB", size_kb), |b| {
            b.iter(|| {
                let _ = xchacha::encode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(payload.clone()).unwrap();
            })
        });

        let ciphertext = xchacha::encode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(payload.clone()).unwrap();

        group.bench_function(format!("decrypt_{}KB", size_kb), |b| {
            b.iter(|| {
                let _ = xchacha::decode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(&ciphertext).unwrap();
            })
        });
    }
    group.finish();
}

fn benchmark_kdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("kdf_performance");
    group.bench_function("key_derivation", |b| {
        b.iter(|| {
            xchacha::benchmark_kdf();
        })
    });
    group.finish();
}

criterion_group!(benches, benchmark_xchacha, benchmark_kdf);
criterion_main!(benches);
