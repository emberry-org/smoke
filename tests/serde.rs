use smoke::messages::Drain;
use smoke::messages::EmbMessage;
use smoke::messages::Source;
use smoke::User;
use tokio::io::BufReader;
use tokio_test::io::Builder;

#[test_log::test(tokio::test)]
async fn stream_test() {
    let msg = EmbMessage::Heartbeat;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[test_log::test(tokio::test)]
async fn stream_test_complex() {
    let msg = EmbMessage::Room(User {
        cert_data: b"Aurelia".to_vec(),
    });
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
}

#[test_log::test(tokio::test)]
async fn stream_test_multiple() {
    let msg = EmbMessage::Room(User {
        cert_data: b"Aurelia".to_vec(),
    });
    let msg2 = EmbMessage::Heartbeat;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg2.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    msg2.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();

    let stream = Builder::new().read(&msg_bytes).read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
}

#[test_log::test(tokio::test)]
async fn stream_test_fragmented() {
    let msg = EmbMessage::Room(User {
        cert_data: b"Aurelia".to_vec(),
    });
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);

    let stream = Builder::new().read(first).read(second).read(third).build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}

#[test_log::test(tokio::test)]
async fn stream_test_fragmented_multi() {
    let msg = EmbMessage::Room(User {
        cert_data: b"Aurelia".to_vec(),
    });
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);

    let stream = Builder::new()
        .read(first)
        .read(second)
        .read(third)
        .read(first)
        .read(second)
        .read(third)
        .build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}

#[tokio::test]
async fn stream_test_fragmented_multi_hybrid() {
    let msg = EmbMessage::Room(User {
        cert_data: b"Aurelia".to_vec(),
    });
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::EMB_MESSAGE_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);

    let stream = Builder::new()
        .read(first)
        .read(second)
        .read(third)
        .read(&msg_bytes)
        .build();
    let mut reader = BufReader::new(stream);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);

    let signal = reader.read_message::<EmbMessage>().await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
}

//TODO Read Error tests
