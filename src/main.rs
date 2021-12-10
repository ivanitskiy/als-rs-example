#[macro_use]
extern crate log;

use std::time::Duration;
use signal_hook::{consts::SIGINT, iterator::Signals};
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

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
use pb::envoy::service::accesslog::v2::{StreamAccessLogsMessage, StreamAccessLogsResponse};

use kafka::error::Error as KafkaError;
use kafka::producer::{Producer, Record, RequiredAcks};

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
        log::info!("Client connected from: {}", req.remote_addr().unwrap());
        let resp = StreamAccessLogsResponse::default();
        let mut stream = req.into_inner();
        while let Some(msg) = stream.next().await {
            let msg = msg?;
            debug!("proto msg = {:#?}", msg);
            let v = match serde_json::to_string_pretty(&msg) {
                Ok(m) => m,
                Err(_) => return  StreamAccessLogsResult::Err(Status::internal("failed to convert to json")),
            };
            debug!("json msg{}",v);

            let broker = "localhost:9092";
            let topic = "my-topic";
            if let Err(e) = produce_message(v.as_bytes(), topic, vec![broker.to_owned()]) {
                error!("Failed producing messages: {}", e);
            }
        }
        let resp = Response::new(resp);
        info!("  about to send response = {:?}", resp);
        StreamAccessLogsResult::Ok(resp)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let als_svc = AlsServer {};
    let addr = "0.0.0.0:50051".parse()?;
    let timeout = Duration::from_secs(30);

    info!("Starting server at {}", addr);

    let mut signals = Signals::new(&[SIGINT])?;
    std::thread::spawn(move || {
        for sig in signals.forever() {
            info!("Received signal {:?}", sig);
            std::process::exit(128 + SIGINT);
        }
    });

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
    info!("some_other_interceptor= {:#?}", request);
    Ok(request)
}

fn als_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    // println!("als_interceptor");
    tracing::event!(Level::INFO, "started processing request");
    Ok(request)
}

// https://github.com/kafka-rust/kafka-rust/blob/master/examples/example-produce.rs
fn produce_message<'a, 'b>(
    data: &'a [u8],
    topic: &'b str,
    brokers: Vec<String>,
) -> Result<(), KafkaError> {
    info!("About to publish a message at {:?} to: {}", brokers, topic);

    // ~ create a producer. this is a relatively costly operation, so
    // you'll do this typically once in your application and re-use
    // the instance many times.
    let mut producer = Producer::from_hosts(brokers)
        // ~ give the brokers one second time to ack the message
        .with_ack_timeout(Duration::from_secs(1))
        // ~ require only one broker to ack the message
        .with_required_acks(RequiredAcks::One)
        // ~ build the producer with the above settings
        .create()?;

    // ~ now send a single message.  this is a synchronous/blocking
    // operation.

    // ~ we're sending 'data' as a 'value'. there will be no key
    // associated with the sent message.

    // ~ we leave the partition "unspecified" - this is a negative
    // partition - which causes the producer to find out one on its
    // own using its underlying partitioner.
    producer.send(&Record {
        topic,
        partition: -1,
        key: (),
        value: data,
    })?;

    // ~ we can achieve exactly the same as above in a shorter way with
    // the following call
    producer.send(&Record::from_value(topic, data))?;

    Ok(())
}