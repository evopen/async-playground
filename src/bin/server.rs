// use async_std::net::TcpListener;
// use async_std::task;

use std::net::SocketAddr;
use std::time::Duration;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};

use tokio::io::{stdin, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use tokio::task;
use tokio::time::sleep;
use tokio_tungstenite::{tungstenite, WebSocketStream};
use tungstenite::Message;

async fn handle_outgoing(
    mut write_stream: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut rx: tokio::sync::watch::Receiver<String>,
) {
    loop {
        let mut s = String::new();
        if rx.changed().await.is_ok() {
            s.clone_from(&*rx.borrow());
            println!("Sending {}", &s);
            write_stream.send(Message::Text(s)).await.unwrap();
        }
    }

    println!("handle outgoing finished");
}

async fn handle_incoming(read_stream: SplitStream<WebSocketStream<TcpStream>>) {
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

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    rx: tokio::sync::watch::Receiver<String>,
) {
    let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
    println!("connected to {:?}", addr);

    let (write, read) = ws.split();

    let _ = tokio::join!(handle_incoming(read), handle_outgoing(write, rx));
}

async fn read_stdin(tx: tokio::sync::watch::Sender<String>) {
    println!("read_stdin in thread {:?}", std::thread::current().id());
    let stdin = tokio::io::stdin();
    let mut buf_reader = tokio::io::BufReader::new(stdin);
    loop {
        let mut input = String::new();
        buf_reader.read_line(&mut input).await.unwrap();
        println!("got{}", &input);
        tx.send(input.clone()).unwrap();
    }
}
async fn run() {
    let listener = TcpListener::bind(("127.0.0.1", 2845)).await.unwrap();
    println!("listening on {}", 2845);

    let (stdin_tx, stdin_rx) = tokio::sync::watch::channel(String::new());
    task::spawn(read_stdin(stdin_tx));

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        let rx = stdin_rx.clone();
        tokio::spawn(handle_connection(stream, addr, rx));
    }
}

fn main() {
    println!(env!("CARGO_BIN_NAME"));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run());
}
