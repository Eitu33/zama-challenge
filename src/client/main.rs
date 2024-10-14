use std::fs;

use clap::{Parser, Subcommand};
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Request;
use hyper::Uri;
use hyper_util::rt::TokioIo;
use mtree::MerkleTree;
use mtree::Order;
use tokio::net::TcpStream;

#[derive(Parser)]
struct Args {
    url: String,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Post { directory: String },
    Get { file_index: u64 },
}

fn post_request(
    url: Uri,
    directory: &str,
) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let dir = fs::read_dir(directory)?;

    for entry in dir {
        let path = entry?.path();
        files.push(fs::read(&path)?);
    }

    if let Some(root) = MerkleTree::new(&files).root() {
        fs::write("root", root)?;
    }

    let bytes = Bytes::from(bincode::serialize(&files)?);
    Ok(Request::post(url).body(Full::new(bytes))?)
}

fn get_request(
    url: Uri,
    file_index: u64,
) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error>> {
    let bytes = Bytes::from(bincode::serialize(&file_index)?);
    Ok(Request::get(url).body(Full::new(bytes))?)
}

type GetReqResponse = (Option<Vec<u8>>, Vec<(Vec<u8>, Order)>);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let url: Uri = args.url.parse::<hyper::Uri>()?;
    let command = args.command;
    let stream = TcpStream::connect(url.to_string()).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let req = match &command {
        Command::Post { directory } => post_request(url, directory),
        Command::Get { file_index } => get_request(url, *file_index),
    }?;

    let mut res = sender.send_request(req).await?;
    println!("Response status: {}", res.status());

    if matches!(command, Command::Get { file_index: _ }) {
        let frame = res.frame().await.unwrap()?;
        let bytes = frame.data_ref().unwrap();
        let (file, proof): GetReqResponse = bincode::deserialize(bytes)?;
        if let Some(content) = file {
            let root = MerkleTree::root_from_proof(&content, proof);
            let saved = fs::read("root")?;

            if root == saved {
                println!("Coherent");
            } else {
                eprintln!("Proof result and local save are different");
            }
        } else {
            eprintln!("There is no file corresponding to this index");
        }
    }

    Ok(())
}
