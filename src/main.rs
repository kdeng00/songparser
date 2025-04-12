use std::error::Error;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("API calling service listening on 127.0.0.1:8080");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut buffer = [0; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            break; // Connection closed
        }

        let request_data = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
        println!("Received request: {}", request_data);
    }

    Ok(())
}
