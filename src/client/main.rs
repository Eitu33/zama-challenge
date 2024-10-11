use std::fs;

use clap::Parser;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Request;
use hyper::Uri;
use hyper_util::rt::TokioIo;
use mtree::MerkleTree;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Parser)]
enum Command {
    Post { directory: String },
    Get { file_index: u64 },
}

fn post_request(
    url: Uri,
    directory: String,
) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let dir = fs::read_dir(directory).unwrap();
    for path in dir {
        files.push(fs::read(path?.path())?);
    }
    fs::write("root", MerkleTree::new(&files).root().unwrap()).unwrap();
    let bytes = Bytes::from(bincode::serialize(&files).unwrap());
    Ok(Request::post(url).body(Full::new(bytes))?)
}

fn get_request(
    url: Uri,
    file_index: u64,
) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
    let bytes = Bytes::from(bincode::serialize(&file_index).unwrap());
    Ok(Request::post(url).body(Full::new(bytes))?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse();
    let url = "127.0.0.1:3000".parse::<hyper::Uri>()?;
    let stream = TcpStream::connect(url.to_string()).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let req = match command {
        Command::Post { directory } => post_request(url, directory),
        Command::Get { file_index } => get_request(url, file_index),
    }?;

    let mut res = sender.send_request(req).await?;
    println!("Response status: {}", res.status());

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(chunk).await?;
        }
    }

    Ok(())
}
