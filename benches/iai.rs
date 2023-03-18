use std::{hint::black_box, io::Cursor};

use smoke::messages::{Drain, Source};
use smoke::Signal;
use tokio::io::BufReader;
use tokio::runtime::Runtime;
use tokio_test::io::Builder;

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

    let _signal = black_box(reader.read_message::<Signal>().await);
}

async fn encode_decode_multiple_fragmented(&quantity: &u64) {
    let mut reader = encode_decode_multiple_fragmented_setup(&quantity).await;
    let mut reader = BufReader::new(reader.build());

    for _ in 0..quantity {
        let _signal = black_box(reader.read_message::<Signal>().await);
        //let _signal = black_box(Signal::recv_with(&mut reader).await);
    }
}

async fn encode_decode_multiple_fragmented_setup(&quantity: &u64) -> Builder {
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

    builder
}

iai::main!(cursor_single_test, fragmented_test, setup, combined_setup);

fn fragmented_test() {
    let runtime = setup();

    runtime.block_on(encode_decode_multiple_fragmented(black_box(&10)));
}

fn cursor_single_test() {
    let runtime = setup();

    runtime.block_on(encode_decode_cursor());
}

#[inline]
fn setup() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

fn combined_setup() {
    let runtime = setup();
    runtime.block_on(encode_decode_multiple_fragmented_setup(&10));
}
