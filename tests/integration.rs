use smoke::Signal;
use smoke::messages::Drain;
use tokio::io::BufReader;
use tokio_test::io::Builder;

#[tokio::test]
async fn stream_test() {
    _ = env_logger::try_init();

    let msg = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_complex() {
    _ = env_logger::try_init();

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_multiple() {
    _ = env_logger::try_init();

    let msg = Signal::Username("Aurelia".to_string());
    let msg2 = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg2.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();
    msg2.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();

    let stream = Builder::new().read(&msg_bytes).read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
}

#[tokio::test]
async fn stream_test_fragmented() {
    _ = env_logger::try_init();

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();
    let mid = msg_bytes.len() >> 1;
    let (first, second) = msg_bytes.split_at(mid);

    let stream = Builder::new().read(first).read(second).build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_fragmented_multi() {
    _ = env_logger::try_init();

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();
    let mid = msg_bytes.len() >> 1;
    let (first, second) = msg_bytes.split_at(mid);

    let stream = Builder::new()
        .read(first)
        .read(second)
        .read(first)
        .read(second)
        .build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_fragmented_multi_hybrid() {
    _ = env_logger::try_init();

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone().serialize_to(&mut msg_bytes, &mut ser_buf).expect("could not serialize").await.unwrap();
    let mid = msg_bytes.len() >> 1;
    let (first, second) = msg_bytes.split_at(mid);

    let stream = Builder::new()
        .read(first)
        .read(second)
        .read(&msg_bytes)
        .build();
    let mut reader = BufReader::new(stream);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}