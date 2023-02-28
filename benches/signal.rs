use std::{hint::black_box, io::Cursor};

use criterion::{criterion_group, criterion_main, Criterion};
use smoke::messages::Drain;
use smoke::Signal;
use tokio::io::BufReader;
use tokio_test::io::Builder;

async fn encode() {
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    Signal::Username(black_box("Aurelia".to_string()))
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .expect("could not send");
}

async fn decode(bytes: &[u8]) {
    let mut reader = BufReader::new(bytes);

    let _signal = black_box(Signal::recv_with(&mut reader).await);
}

async fn encode_decode_mock() {
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    Signal::Username(black_box("Aurelia".to_string()))
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .expect("could not send");

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let _signal = black_box(Signal::recv_with(&mut reader).await);
}

async fn encode_decode_cursor() {
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    Signal::Username(black_box("Aurelia".to_string()))
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .expect("could not send");

    let cursor = Cursor::new(msg_bytes);
    let mut reader = BufReader::new(cursor);

    let _signal = black_box(Signal::recv_with(&mut reader).await);
}

async fn encode_decode_multiple(&quantity: &u64) {
    let mock = {
        let msg = Signal::Username("Aurelia".to_string());
        let mut msg_bytes = Vec::<u8>::new();
        let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
        for _ in 0..quantity {
            msg.clone()
                .serialize_to(&mut msg_bytes, &mut ser_buf)
                .expect("could not serialize")
                .await
                .unwrap();
        }

        Builder::new().read(&msg_bytes).build()
    };

    let mut reader = BufReader::new(mock);

    for _ in 0..quantity {
        let _signal = black_box(Signal::recv_with(&mut reader).await);
    }
}

async fn encode_decode_multiple_unbuffered(&quantity: &u64) {
    let mock = {
        let msg = Signal::Username("Aurelia".to_string());
        let mut msg_bytes = Vec::<u8>::new();
        let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
        let mut builder = Builder::new();
        for _ in 0..quantity {
            msg.clone()
                .serialize_to(&mut msg_bytes, &mut ser_buf)
                .expect("could not serialize")
                .await
                .unwrap();
            builder.read(&msg_bytes);
            msg_bytes.clear();
        }

        builder.build()
    };

    let mut reader = BufReader::new(mock);

    for _ in 0..quantity {
        let _signal = black_box(Signal::recv_with(&mut reader).await);
    }
}

async fn encode_decode_multiple_fragmented(&quantity: &u64) {
    let mock = {
        let msg = Signal::Username("Aurelia".to_string());
        let mut msg_bytes = Vec::<u8>::new();
        let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
        let mut builder = Builder::new();
        for _ in 0..quantity {
            msg.clone()
                .serialize_to(&mut msg_bytes, &mut ser_buf)
                .expect("could not serialize")
                .await
                .unwrap();

            let mid = msg_bytes.len() >> 1;
            let (first, second) = msg_bytes.split_at(mid);

            builder.read(first).read(second);

            msg_bytes.clear();
        }

        builder.build()
    };

    let mut reader = BufReader::new(mock);

    for _ in 0..quantity {
        let _signal = black_box(Signal::recv_with(&mut reader).await);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("raw");

    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    runtime
        .block_on(
            Signal::Username(black_box("Aurelia".to_string()))
                .serialize_to(&mut msg_bytes, &mut ser_buf)
                .expect("could not serialize"),
        )
        .unwrap();
    group.bench_function("encode", |b| b.to_async(&runtime).iter(encode));
    group.bench_with_input("decode", &msg_bytes, |b, i| {
        b.to_async(&runtime).iter(|| decode(i))
    });

    group.finish();

    let mut group = c.benchmark_group("encode_decode");
    group.bench_function("mock", |b| b.to_async(&runtime).iter(encode_decode_mock));
    group.bench_function("cursor", |b| {
        b.to_async(&runtime).iter(encode_decode_cursor)
    });

    group.finish();

    let mut group = c.benchmark_group("multiple_decode");
    const QUANTITY: u64 = 10;

    group.bench_with_input("buffered", &QUANTITY, |b, quantity| {
        b.to_async(&runtime)
            .iter(|| encode_decode_multiple(quantity))
    });

    group.bench_with_input("unbuffered", &QUANTITY, |b, quantity| {
        b.to_async(&runtime)
            .iter(|| encode_decode_multiple_unbuffered(quantity))
    });

    group.bench_with_input("fragmented", &QUANTITY, |b, quantity| {
        b.to_async(&runtime)
            .iter(|| encode_decode_multiple_fragmented(quantity))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
