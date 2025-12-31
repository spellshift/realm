use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use pb::xchacha;
use rand::Rng;

// Define a simple wrapper for Vec<u8> that implements prost::Message
// We can't use Vec<u8> directly as Message because it's not a message, it's a field type usually.
// So we use a dummy message from the codebase or define a simple one.
// Looking at xchacha.rs, it requires T and U to be Message + Default + Send + 'static.

// We will use the `pb::generated::eldritch::File` message if accessible, or we can use a custom one
// if we can derive Message. Since we can't easily run `prost-build` here just for the bench,
// we rely on `pb` exporting the generated messages.
// `pb` exports modules in `src/lib.rs`. Let's assume `pb::eldritch::File` or similar exists.
// Actually, `pb` crate structure needs to be checked.
// `implants/lib/pb/src/lib.rs` likely exposes `generated` modules.

// Let's first check `implants/lib/pb/src/lib.rs` to see what is exported.
// But based on file list `implants/lib/pb/src/generated/eldritch.rs`, it likely contains `File`.

// However, simpler is better. We can define a struct and implement Message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BenchmarkPayload {
    #[prost(bytes = "vec", tag = "1")]
    pub data: Vec<u8>,
}

// We also need a dummy response message type
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
                // We use encode_with_chacha directly as it wraps encrypt_impl
                let _ = xchacha::encode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(payload.clone()).unwrap();
            })
        });

        // For decryption, we first encrypt it to get a valid ciphertext
        let ciphertext = xchacha::encode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(payload.clone()).unwrap();

        group.bench_function(format!("decrypt_{}KB", size_kb), |b| {
            b.iter(|| {
                let _ = xchacha::decode_with_chacha::<BenchmarkPayload, BenchmarkResponse>(&ciphertext).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_xchacha);
criterion_main!(benches);
