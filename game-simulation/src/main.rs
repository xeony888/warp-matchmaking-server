use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use std::collections::HashMap;
use std::process;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio_tungstenite::accept_async;

fn parse_args() -> HashMap<String, String> {
    let args: Vec<String> = std::env::args().collect();
    let mut map = HashMap::new();
    let mut i = 1;
    while i < args.len() {
        if args[i].starts_with("-") && i + 1 < args.len() {
            map.insert(args[i].clone(), args[i + 1].clone());
            i += 2;
        } else {
            eprintln!("Invalid arguments.");
            process::exit(1);
        }
    }

    map
}

#[tokio::main]
async fn main() {
    println!("Running");
    let exit_codes = [1000, 1001, 1002];
    let args = parse_args();

    let port = args.get("-port").unwrap_or_else(|| {
        eprintln!("Missing -port");
        process::exit(1);
    });

    let username1 = args.get("-username1").unwrap_or_else(|| {
        eprintln!("Missing -username1");
        process::exit(1);
    });

    let username2 = args.get("-username2").unwrap_or_else(|| {
        eprintln!("Missing -username2");
        process::exit(1);
    });

    let token1 = args.get("-player1token").unwrap_or_else(|| {
        eprintln!("Missing -player1token");
        process::exit(1);
    });

    let token2 = args.get("-player2token").unwrap_or_else(|| {
        eprintln!("Missing -player2token");
        process::exit(1);
    });

    println!("Port: {}", port);
    println!("Username1: {}, Token1: {}", username1, token1);
    println!("Username2: {}, Token2: {}", username2, token2);

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|_| {
        eprintln!("Failed to bind to {}", addr);
        process::exit(1);
    });

    println!("WebSocket server listening on ws://{}", addr);

    let timeout = tokio::time::sleep(Duration::from_secs(60));

    let server = async {
        loop {
            let (stream, _) = listener.accept().await.expect("Failed to accept connection");

            tokio::spawn(async move {
                let ws_stream = accept_async(stream).await.expect("WebSocket handshake failed");

                println!("New WebSocket connection");

                let (mut write, mut read) = ws_stream.split();
                while let Some(Ok(msg)) = read.next().await {
                    if msg.is_text() || msg.is_binary() {
                        if let Err(e) = write.send(msg).await {
                            eprintln!("Send error: {}", e);
                            break;
                        }
                    }
                }

                println!("Connection closed");
            });
        }
    };

    select! {
        _ = server => {},
        _ = timeout => {
            let code = exit_codes[rand::rng().random_range(0..exit_codes.len())];
            println!("Exiting after 60 seconds with code: {}", code);
            process::exit(code);
        }
    }
}
