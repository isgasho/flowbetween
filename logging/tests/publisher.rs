extern crate flo_logging;
extern crate desync;

use flo_logging::*;
use desync::*;

use std::sync::*;
use std::thread;
use std::time::Duration;

#[test]
fn publish_log_messages_to_subscriber() {
    let log         = LogPublisher::new();
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe(), |messages, new_message| messages.push(new_message.unwrap()));

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(&messages[0].message() == "Hello, world");
    assert!(&messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

#[test]
fn publish_log_messages_to_default() {
    let log         = LogPublisher::new();
    let messages    = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages), log.subscribe_default(), |messages, new_message| messages.push(new_message.unwrap()));

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(&messages[0].message() == "Hello, world");
    assert!(&messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}

#[test]
fn no_messages_to_default_with_subscriber() {
    let log                 = LogPublisher::new();
    let messages_default    = Arc::new(Desync::new(vec![]));
    let messages_nondefault = Arc::new(Desync::new(vec![]));

    pipe_in(Arc::clone(&messages_default), log.subscribe_default(), |messages, new_message| messages.push(new_message.unwrap()));
    pipe_in(Arc::clone(&messages_nondefault), log.subscribe(), |messages, new_message| messages.push(new_message.unwrap()));

    log.log("Hello, world");
    log.log("... goodbye, world :-(");

    let messages_nondefault = messages_nondefault.sync(|messages_nondefault| messages_nondefault.clone());
    let messages_default    = messages_default.sync(|messages_default| messages_default.clone());    

    assert!(messages_nondefault.len() != 0);
    assert!(&messages_nondefault[0].message() == "Hello, world");
    assert!(&messages_nondefault[1].message() == "... goodbye, world :-(");
    assert!(messages_nondefault.len() == 2);

    assert!(messages_default.len() == 0);
}

#[test]
fn stream_between_logs() {
    let src         = LogPublisher::new();
    let tgt         = LogPublisher::new();
    let messages    = Arc::new(Desync::new(vec![]));

    // Result is messages from target
    pipe_in(Arc::clone(&messages), tgt.subscribe(), |messages, new_message| messages.push(new_message.unwrap()));

    // Target relays logs from src
    tgt.stream(src.subscribe());

    // Send messages to src
    src.log("Hello, world");
    src.log("... goodbye, world :-(");

    thread::sleep(Duration::from_millis(20));   // TODO: arrange things so that propagation is instant somehow
    let messages    = messages.sync(|messages| messages.clone());

    assert!(messages.len() != 0);
    assert!(&messages[0].message() == "Hello, world");
    assert!(&messages[1].message() == "... goodbye, world :-(");
    assert!(messages.len() == 2);
}
