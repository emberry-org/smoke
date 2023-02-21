use smoke::Signal;
use tokio::io::BufReader;
use tokio_test::io::Builder;

#[tokio::test]
async fn stream_test() {
    let msg = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    msg.clone().send_with(&mut msg_bytes).await.unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let mut buf = Vec::with_capacity(1024);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_complex() {
    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    msg.clone().send_with(&mut msg_bytes).await.unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let mut buf = Vec::with_capacity(1024);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_multiple() {
    let msg = Signal::Username("Aurelia".to_string());
    let msg2 = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    msg2.clone().send_with(&mut msg_bytes).await.unwrap();
    msg.clone().send_with(&mut msg_bytes).await.unwrap();
    msg2.clone().send_with(&mut msg_bytes).await.unwrap();

    let stream = Builder::new().read(&msg_bytes).read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let mut buf = Vec::with_capacity(1024);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = Signal::recv_with(&mut reader, &mut buf).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
}
