use tonic::Request;
use tracing::info;

pub mod greeter {
    tonic::include_proto!("greeter");
}

use greeter::greeter_client::GreeterClient;
use greeter::HelloRequest;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut client = GreeterClient::connect("http://127.0.0.1:50051").await?;

    // unary
    let resp = client
        .say_hello(Request::new(HelloRequest { name: "world".into() }))
        .await?;
    info!(msg = %resp.into_inner().message, "unary reply");

    // server stream
    let mut stream = client
        .say_hello_stream(Request::new(HelloRequest { name: "world".into() }))
        .await?
        .into_inner();

    while let Some(reply) = stream.message().await? {
        info!(msg = %reply.message, "streamed");
    }
    Ok(())
}
