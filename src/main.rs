pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));

}


use crate::pb::envoy::service::accesslog::v2::access_log_service_server::{
    AccessLogService, AccessLogServiceServer,
};
use tokio_stream::StreamExt;
use tonic::{
    transport::Server, Request, Response, Status, Streaming,
    service::interceptor
};

use tracing::Level;

use tower_http::{
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer, DefaultOnFailure},
    LatencyUnit,
};

use pb::envoy::service::accesslog::v2::{StreamAccessLogsMessage, StreamAccessLogsResponse};

type StreamAccessLogsResult<T> = Result<Response<T>, Status>;

#[derive(Debug)]
pub struct AlsServer {}

#[tonic::async_trait]
impl AccessLogService for AlsServer {
    async fn stream_access_logs(
        &self,
        req: Request<Streaming<StreamAccessLogsMessage>>,
    ) -> StreamAccessLogsResult<StreamAccessLogsResponse> {
        // StreamAccessLogsResult::Err(Status::unimplemented("not implemented"))
        println!("Client connected from: {}", req.remote_addr().unwrap());
        let resp = StreamAccessLogsResponse::default();
        let mut stream = req.into_inner();
        while let Some(msg) = stream.next().await {
            let msg = msg?;
            println!("proto msg = {:#?}", msg);
            let v = match serde_json::to_string_pretty(&msg) {
                Ok(m) => m,
                Err(_) => return  StreamAccessLogsResult::Err(Status::internal("failed to convert to json")),
            };
            println!("json msg{}",v);
        }
        let resp = Response::new(resp);
        println!("  about to send response = {:?}", resp);
        StreamAccessLogsResult::Ok(resp)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let als_svc = AlsServer {};
    let addr = "[::1]:50051".parse()?;
    let timeout = std::time::Duration::from_secs(30);

    let mut builder = Server::builder();
    builder.timeout(timeout);

    builder
        // TraceLayer::new_for_grpc doesn't works
        .layer(
            TraceLayer::new_for_grpc()
                // .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnRequest::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros), // on so on for `on_eos`, `on_body_chunk`, and `on_failure`
                ),
        )
        .layer(interceptor(some_other_interceptor))
        .add_service(AccessLogServiceServer::with_interceptor(als_svc, als_interceptor))
        .serve(addr)
        .await
        .unwrap();
    Ok(())
}

fn some_other_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    println!("some_other_interceptor= {:#?}", request);
    Ok(request)
}

fn als_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    // println!("als_interceptor");
    tracing::event!(Level::INFO, "started processing request");
    Ok(request)
}