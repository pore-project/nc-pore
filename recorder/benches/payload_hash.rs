use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use recorder::artifact::PayloadHash;

const PAYLOAD_SIZES: &[usize] = &[64 * 1024, 1_048_576, 10_485_760, 67_108_864];

fn benchmark_payload_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_hash_overhead");

    for &size in PAYLOAD_SIZES {
        let payload = vec![0x5a; size];
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(BenchmarkId::new("copy_only", size), &payload, |b, data| {
            b.iter(|| black_box(data.clone()));
        });

        group.bench_with_input(
            BenchmarkId::new("copy_and_hash", size),
            &payload,
            |b, data| {
                b.iter(|| {
                    let copied = data.clone();
                    black_box(PayloadHash::from_bytes(&copied));
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("hash_only", size), &payload, |b, data| {
            b.iter(|| black_box(PayloadHash::from_bytes(data)));
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_payload_hash);
criterion_main!(benches);
