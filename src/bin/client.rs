use std::net::SocketAddr;
use std::time::Duration;

use async_tungstenite::tungstenite::Message;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tokio::time::sleep;
use tokio_tungstenite::{tungstenite, WebSocketStream};

async fn handle_outgoing(mut write_stream: SplitSink<WebSocketStream<TcpStream>, Message>) {
    println!("{:?}", std::thread::current().id());

    loop {
        sleep(Duration::from_secs(1)).await;
        write_stream
            .send(Message::Text("do you know".into()))
            .await
            .unwrap();
        println!("sent");
    }

    println!("handle outgoing finished");
}

async fn handle_incoming(read_stream: SplitStream<WebSocketStream<TcpStream>>) {
    println!("{:?}", std::thread::current().id());
    read_stream
        .for_each(|msg| async {
            match msg.unwrap() {
                Message::Text(s) => {
                    println!("receive {}", s);
                }
                Message::Binary(_) => {}
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {}
            }
        })
        .await;
    println!("handle incoming finished");
}
async fn run() {
    let (ws, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:2845")
        .await
        .unwrap();

    let (write, read) = ws.split();

    let _ = tokio::join!(handle_incoming(read), handle_outgoing(write));
}

fn main() {
    // let rt = tokio::runtime::Runtime::new().unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run());
}
