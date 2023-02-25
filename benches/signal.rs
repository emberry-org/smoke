use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use smoke::Signal;
use tokio::io::BufReader;
use tokio_test::io::Builder;

async fn encode() {
    let mut msg_bytes = Vec::<u8>::new();
    Signal::Username("Aurelia".to_string()).send_with(&mut msg_bytes).await.unwrap();
}

async fn encode_decode() {
    let mut msg_bytes = Vec::<u8>::new();
    Signal::Username("Aurelia".to_string()).send_with(&mut msg_bytes).await.unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();

    let _signal = black_box(Signal::recv_with(&mut reader, &mut buf).await);
}

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let mut group = c.benchmark_group("signal");

    group.bench_function("encode", |b| b.to_async(&runtime).iter(encode));
    group.bench_function("encode_decode", |b| {
        b.to_async(&runtime).iter(encode_decode)
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
