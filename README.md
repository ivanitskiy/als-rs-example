This is an example of using rust to build envoy access log server, convert protobuf messages to json format, send messages to kafka broker and read messages from kafka broker (pretty printed json).

---

Start kafka compatible broker [redpanda](https://github.com/vectorizedio//blob/dev/tools/docker/README.md) in docker:

    docker-compose up -d redpanda

Start als server:

    cargo run --bin als-rs


Start kafka client:

    cargo run --bin client --   --brokers localhost:9092 --group-id client2 --topics my-topic


Send RPC messages to the als server, and watch for the kafka client.


Optionally, install [bloomrpc](https://github.com/bloomrpc/bloomrpc) to sends gRPC messages to *als-rs* server without envoy server running for local testing only. Import proto messages into *bloomrpc* and you can test it without running envoy