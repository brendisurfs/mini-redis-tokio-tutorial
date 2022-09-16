//---------------NOTES -------------------
// 1. Variables are not moved into async blocks.
// 2. just because a variables lifetime is 'static, doesnt mean it lives forever.
// 3. Variables MUST have a static lifetime.
// 4. If a piece of data needs to be accessed from more than one task concurrently,
//    it must be shared using the Arc synchronization primitive (or some alternative).
//  5. Tasks spawned by tokio::spawn must implement Send.

// ------------ project -----------------
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // bind the listener to the specific addr we want.
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("could not bind to port");

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("could not accept incoming connection");

        // 1. a Tokio task is an async green thread.
        //  - Created by passing an async block to tokio::spawn.
        //  - Returns a join handle, so that the caller can interact with that spawned task.
        tokio::spawn(async move {
            // a new task is spawned for each inbound socket.
            // The socket is moved to the new task and processed there.
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    let mut db = HashMap::new();

    // the `Connection` lets us read/write redis **frames** instead of byte streams.
    // the Connection type is defined by mini-redis.
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd_from_frame = Command::from_frame(frame).expect("could not read from frame");
        let response = match cmd_from_frame {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // Frame::Bulk will expect data to be type "Bytes".
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
