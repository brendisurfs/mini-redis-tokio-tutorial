use mini_redis::{Connection, Frame};
use std::io::Result;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // bind the listener to the specific addr we want.
    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("could not bind to port");

    loop {
        // the second item containst he IP and port of the new connection.
        let (socket, _) = listener
            .accept()
            .await
            .expect("could not accept incoming connection");

        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    // the `Connection` lets us read/write redis **frames** instead of byte streams.
    // the Connection type is defined by mini-redis.

    let mut connection = Connection::new(socket);

    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("Got: {frame:?}");

        let err_response = Frame::Error("unimplemented".to_string());
        connection
            .write_frame(&err_response)
            .await
            .expect("could not write frame");
    }
}
