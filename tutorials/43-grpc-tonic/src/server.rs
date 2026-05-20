use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod greeter {
    tonic::include_proto!("greeter");
}

use greeter::greeter_server::{Greeter, GreeterServer};
use greeter::{HelloReply, HelloRequest};

#[derive(Default)]
struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = req.into_inner().name;
        info!(%name, "unary call");
        Ok(Response::new(HelloReply { message: format!("Hello, {name}!") }))
    }

    type SayHelloStreamStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<HelloReply, Status>> + Send>>;

    async fn say_hello_stream(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        let name = req.into_inner().name;
        info!(%name, "server stream");
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        tokio::spawn(async move {
            for i in 0..5 {
                let _ = tx.send(Ok(HelloReply {
                    message: format!("Hello {name} #{i}"),
                })).await;
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });
        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:50051".parse()?;
    info!("Greeter listening on {addr}");
    Server::builder()
        .add_service(GreeterServer::new(MyGreeter))
        .serve(addr)
        .await?;
    Ok(())
}
