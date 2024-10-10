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
use tokio::net::TcpListener;

async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    match *req.method() {
        Method::GET => Ok(Response::new(Full::new("Call mtree here\n".into()).boxed())),
        Method::POST => {
            let data = req.into_body().frame().await.unwrap()?.into_data().unwrap();
            let files: Vec<Vec<u8>> = bincode::deserialize(&data).unwrap();
            for (i, file) in files.iter().rev().enumerate() {
                fs::write(format!("data/server/{}.txt", i), file).unwrap();
            }
            Ok(Response::new(Full::new("Saved\n".into()).boxed()))
        }
        _ => {
            let mut not_found = Response::new(Empty::new().boxed());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(echo))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
