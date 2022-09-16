//---------------NOTES -------------------
// 1. Variables are not moved into async blocks.
// 2. just because a variables lifetime is 'static, doesnt mean it lives forever.
// 3. Variables MUST have a static lifetime.
// 4. If a piece of data needs to be accessed from more than one task concurrently,
//    it must be shared using the Arc synchronization primitive (or some alternative).
//  5. Tasks spawned by tokio::spawn must implement Send.

// ------------ project -----------------
use bytes::Bytes;
use log::info;
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

// simpler to read + easier for me to remember later.
type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
    // bind the listener to the specific addr we want.
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("could not bind to port");

    println!("starting server");

    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("could not accept incoming connection");

        // 1. a Tokio task is an async green thread.
        //  - Created by passing an async block to tokio::spawn.
        //  - Returns a join handle, so that the caller can interact with that spawned task.
        info!("accepted incoming connection");
        let db = db.clone();
        tokio::spawn(async move {
            // a new task is spawned for each inbound socket.
            // The socket is moved to the new task and processed there.
            process(socket, db).await;
        });
    }
}

// takes in the socket and the *shared handle* to the hash map as an arg.
async fn process(socket: TcpStream, db: Db) {
    // THIS FEELS GOOD LIKE IM WRITING SOME REASON OR SOMETHING LOCAL USES.
    use mini_redis::Command::{self, Get, Set};

    // the `Connection` lets us read/write redis **frames** instead of byte streams.
    // the Connection type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd_from_frame = Command::from_frame(frame).expect("could not read from frame");
        let response = match cmd_from_frame {
            Set(cmd) => {
                // NOTE: this is important for mutating the shared data.
                let mut db = db.lock().expect("could not lock arc db");
                let cmd_key = cmd.key().to_string();
                let cmd_value = cmd.value().clone();

                db.insert(cmd_key, cmd_value);

                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().expect("could not lock arc db");
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented: {cmd:?}"),
        };

        connection
            .write_frame(&response)
            .await
            .expect("could not write frame");
    }
}

// since we have little contention with synchronous mutex, mutex sharding is fine to use.
// type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;
//
// fn new_sharded_db(num_shards: usize) -> ShardedDb {
//     let mut db = Vec::with_capacity(num_shards);
//     for _ in 0..num_shards {
//         db.push(Mutex::new(HashMap::new()));
//     }
//     Arc::new(db)
// }
