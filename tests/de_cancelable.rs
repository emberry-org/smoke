use smoke::messages::Drain;
use smoke::messages::Source;
use smoke::Signal;

use tokio::io::BufReader;
use tokio::time::Duration;

use tokio_test::assert_pending;
use tokio_test::assert_ready;
use tokio_test::io::Builder;

#[test_log::test(tokio::test)]
async fn stream_test() {
    let msg = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test)]
async fn stream_test_complex() {
    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();

    let stream = Builder::new().read(&msg_bytes).build();
    let mut reader = BufReader::new(stream);

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;

    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test)]
async fn stream_test_multiple() {
    let msg = Signal::Username("Aurelia".to_string());
    let msg2 = Signal::Kap;
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
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

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok());
    assert_eq!(signal.unwrap(), msg2);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test)]
async fn stream_test_fragmented() {
    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);

    let stream = Builder::new().read(first).read(second).read(third).build();
    let mut reader = BufReader::new(stream);

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;

    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test)]
async fn stream_test_fragmented_multi() {
    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
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

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test)]
async fn stream_test_fragmented_multi_hybrid() {
    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
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

    let mut agg = Vec::new();
    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());

    let signal = reader.read_message_cancelable::<Signal>(&mut agg).await;
    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test(start_paused = true))]
async fn stream_test_fragmented_canceled_early() {
    const WAIT: Duration = Duration::from_secs(1);

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);
    let first_and_second: Vec<u8> = first
        .iter()
        .copied()
        .chain(second.iter().copied())
        .collect();

    let stream = Builder::new()
        .read(first)
        .wait(WAIT)
        // cancel on first
        .read(second)
        .wait(WAIT)
        .read(third)
        .build();
    let mut reader = BufReader::new(stream);

    let mut agg = Vec::new();
    let mut task = tokio_test::task::spawn(reader.read_message_cancelable::<Signal>(&mut agg));
    assert_pending!(task.poll(), "poll 1 should NOT yield result");
    // simulate cancelation like in a select! by dropping the future
    drop(task);
    assert_eq!(
        &agg, first,
        "agg should have parts 1 of the message after poll 1"
    );

    let unsafe_agg: *const Vec<u8> = &agg as *const Vec<u8>;
    let mut task = tokio_test::task::spawn(reader.read_message_cancelable::<Signal>(&mut agg));
    // poll 2 more times to get the signal

    // sleep for the io delay
    tokio::time::sleep(WAIT).await;
    assert_pending!(task.poll(), "poll 2 should NOT yield result");
    assert_eq!(
        unsafe { &*unsafe_agg },
        &first_and_second,
        "agg should have parts 1 and 2 of the message after poll 2"
    );

    // sleep for the io delay
    tokio::time::sleep(WAIT).await;
    let signal = assert_ready!(task.poll(), "third poll should yield result");

    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

#[test_log::test(tokio::test(start_paused = true))]
async fn stream_test_fragmented_canceled_late() {
    const WAIT: Duration = Duration::from_secs(1);

    let msg = Signal::Username("Aurelia".to_string());
    let mut msg_bytes = Vec::<u8>::new();
    let mut ser_buf = [0u8; smoke::messages::signal::MAX_SIGNAL_BUF_SIZE];
    msg.clone()
        .serialize_to(&mut msg_bytes, &mut ser_buf)
        .expect("could not serialize")
        .await
        .unwrap();
    let (first, second) = msg_bytes.split_at(msg_bytes.len() / 3);
    let (second, third) = second.split_at(second.len() / 2);
    let first_and_second: Vec<u8> = first
        .iter()
        .copied()
        .chain(second.iter().copied())
        .collect();

    let stream = Builder::new()
        .read(first)
        .wait(WAIT)
        .read(second)
        .wait(WAIT)
        // cancel on second
        .read(third)
        .build();

    let mut reader = BufReader::new(stream);

    let mut agg = Vec::new();
    let unsafe_agg: *const Vec<u8> = &agg as *const Vec<u8>;
    let mut task = tokio_test::task::spawn(reader.read_message_cancelable::<Signal>(&mut agg));
    assert_pending!(task.poll(), "poll 1 should NOT yield result");
    assert_eq!(
        unsafe { &*unsafe_agg },
        &first,
        "agg should have parts 1 of the message after poll 1"
    );

    // sleep for the io delay
    tokio::time::sleep(WAIT).await;
    assert_pending!(task.poll(), "poll 2 should NOT yield result");
    // simulate cancelation like in a select! by dropping the future
    drop(task);
    assert_eq!(
        &agg, &first_and_second,
        "agg should have parts 1 and 2 of the message after poll 2"
    );

    let mut task = tokio_test::task::spawn(reader.read_message_cancelable::<Signal>(&mut agg));
    // poll one final time to get signal

    // sleep for the io delay
    tokio::time::sleep(WAIT).await;
    let signal = assert_ready!(task.poll(), "third poll should yield result");

    assert!(signal.is_ok(), "{:?}", signal.unwrap_err());
    assert_eq!(signal.unwrap(), msg);
    assert!(agg.is_empty());
}

//TODO Read Error tests
