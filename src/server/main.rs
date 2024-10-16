use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt};
use http_body_util::{Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use mtree::MerkleTree;
use tokio::net::TcpListener;

async fn serve(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    match *req.method() {
        Method::GET => {
            let data = req.into_body().frame().await.unwrap()?.into_data().unwrap();
            let i: usize = bincode::deserialize(&data).unwrap();

            let mut files = Vec::new();
            let mut paths: Vec<_> = fs::read_dir("data/server")
                .unwrap()
                .map(|r| r.unwrap().path())
                .collect();
            paths.sort();

            for path in paths {
                files.push(fs::read(path).unwrap());
            }

            let mtree = MerkleTree::new(&files);
            let proof = mtree.merkle_proof(i);
            let load = bincode::serialize(&(files.get(i), proof)).unwrap();

            Ok(Response::new(Full::new(load.into()).boxed()))
        }
        Method::POST => {
            let data = req.into_body().frame().await.unwrap()?.into_data().unwrap();
            let files: Vec<Vec<u8>> = bincode::deserialize(&data).unwrap();

            for (i, file) in files.iter().enumerate() {
                fs::write(format!("data/server/{}.txt", i), file).unwrap();
            }

            Ok(Response::new(Empty::new().boxed()))
        }
        _ => {
            let mut not_found = Response::new(Empty::new().boxed());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    fs::create_dir_all("data/server")?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(serve))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
