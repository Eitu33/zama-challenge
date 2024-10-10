use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Request;
use hyper_util::rt::TokioIo;
use mtree::MerkleTree;
use std::fs;
use tokio::io::{self, AsyncWriteExt as _};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "127.0.0.1:3000".parse::<hyper::Uri>()?;
    let host = url.host().unwrap();
    let port = url.port_u16().unwrap_or(80);
    let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let mut files = Vec::new();
    let dir = fs::read_dir("data/client").unwrap();
    for path in dir {
        files.push(fs::read(path?.path())?);
    }
    fs::write("data/root", MerkleTree::new(&files).root().unwrap()).unwrap();
    let bytes = bincode::serialize(&files).unwrap();

    let req = Request::post(url)
        .body(Full::new(Bytes::from(bytes)))
        .unwrap();

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
