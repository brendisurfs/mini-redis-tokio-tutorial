// ------------NOTES ----------------
// How the heck are we going to use client across two(or more) threads?
// implementing Copy is out, we cannot use sync::Mutex as await would need to
// be called with the lock held.
// Answer: we will use message passing!
// there are many channels within Tokio:
// 1. mpsc:
//      - multi-producer, single-consumer channel.
//      - Many values can be sent.
// 2. oneshot:
//      - single-producer, single consumer.
//      - a single value can be sent.
// 3. broadcast:
//      - multi-producer, multi-consumer.
//       - many values can be sent, and each receiver sees every value.
// 4. watch:
//       - single producer, multi-consumer.
//       - Many values can be sent, but no history is kept,
//         meaning the receiver only see the most recent value.
//
//
//
//
//
use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
}

#[tokio::main]
async fn main() {
    // create a new channel with cap. at 32.
    let (tx, mut rx) = mpsc::channel(32);
    let tx_two = tx.clone();
    // the mpsc channel is used to send comamnds to the task managing
    // the redis connection.
    // the multi-producer allows messages to be sent from many tasks.

    let mut client = client::connect("127.0.0.1:6379")
        .await
        .expect("could not connet to client");

    let thread_one = tokio::spawn(async move {
        tx_two.send("sending from the first handle").await;
        // let response = client.get("hello").await;
    });

    let thread_two = tokio::spawn(async {
        tx_two.send("sending from the second handle").await;
        // client.set("foo", "bar".into()).await;
    });
}
