# Vehicle Information Service

This is an implementation of the [Vehicle Information Service standard](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html).
The goal of the implementation is to make it simple to implement VIS actions for vehicle signals.
E.g. `Subscribe`, `Get` and `Set`,  while efficiently handling Websocket connections, clients and subscriptions.

# NOTICE

Before you use the program in productive use, please take all necessary precautions,
e.g. testing and verifying the program with regard to your specific use.
The program was tested solely for our own use cases, which might differ from yours.

# Example

For a full example, that is guaranteed to build, check the `examples/` folder.
A brief glimpse is given here:

```rust

#[macro_use]
extern crate log;

use actix::prelude::*;
use actix_web::server;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio_socketcan;

use vehicle_information_service::{KnownError, Router, Set, SignalManager, UpdateSignal};

const PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID: &str = "Private.Example.SocketCan.Last.Frame.Id";

fn main() {
    env_logger::init();

    let sys = actix::System::new("vis-example");

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 14430);

    server::new(|| {
        let app = Router::start();

        let can_id_stream = tokio_socketcan::CANSocket::open("vcan0")
            .expect("Failed to initialize CanSocket")
            .map(|frame| frame.id());

        app.state()
            .spawn_stream_signal_source(PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID.into(), can_id_stream);
    })
    .bind(socket_addr)
    .unwrap()
    .start();

    let _ = sys.run();
}
```

# Running
The "server" example demonstrates a simple VIS server that uses [SocketCAN](https://www.kernel.org/doc/html/v4.17/networking/can.html)
as a signal source, as well as a simple counter signal source .
You may run the example by setting up a vcan and running the example server executable.

```
# Set up vcan
sudo modprobe vcan && \
  sudo ip link add dev vcan0 type vcan && \
  sudo ip link set up vcan0

# Run executable
RUST_LOG=debug cargo +nightly run --example server -- --port 14430 --can vcan0
```

## Websocket Client
```
# Open websocket connection with wscat
wscat -c "localhost:14430"
{ "action": "subscribe", "path": "Private.Example.Interval", "requestId": "d2c7c1a2-f5aa-4fce-9d34-3323fdf20236"}
{ "action": "subscribe", "path": "Private.Example.Interval", "requestId": "1005", "filters": { "range": { "above": 5, "below": 10 } }}
{ "action": "subscribe", "path": "Private.Example.Interval", "requestId": "1006", "filters": { "minChange": "abc" } }
{ "action": "subscribe", "path": "Private.Example.Interval", "requestId": "1007", "filters": { "interval": 3, "range": { "above": 10, "below": 20 } } }
{ "action": "unsubscribe", "subscriptionId": "4afdcdce-d5f9-48de-8f8e-1250e53b2dcd", "requestId": "1008"}
{ "action": "unsubscribeAll", "requestId": "1009"}
{ "action": "get", "path": "Private.Example.Interval", "requestId": "1010"}
{ "action": "get", "path": "Private.Example.SocketCan.Last.Frame.Id", "requestId": "1011"}
{ "action": "subscribe", "path": "Private.Example.SocketCan.Last.Frame.Id", "requestId": "1012"}
```

## Output
```
$ wscat -c "localhost:14430"

connected (press CTRL+C to quit)
> { "action": "subscribe", "path": "Private.Example.Interval", "requestId": "1004"}

< {"action":"subscribe","requestId":"1004","subscriptionId":"2b1c7a38-0c6d-4eb3-a5cb-352245bfd596","timestamp":1511351899913}

< {"action":"subscriptionNotification","subscriptionId":"2b1c7a38-0c6d-4eb3-a5cb-352245bfd596","value": 1, "timestamp":1511351902760}
```

## Limitations
- For now this implementation does not support path wildcards.
- The `getMetadata` action is currently unsupported.
- The `authorize` action is currently unsupported.

# Tests
Unit tests can be executed using

```
cargo test
```

Integration tests are located in the vehicle-information-service-client.
Currently the server example has to be started manually before running the integration tests.
```
cd vehicle-information-service && cargo run --example server -- --port 14430 --can vcan0 &
cd ../vehicle-information-service-client && cargo test
```

# Code of Conduct

Please read our [Code of Conduct](https://github.com/Daimler/daimler-foss/blob/master/CODE_OF_CONDUCT.md) as it is our base for interaction.

# Provider Information

Please visit <https://www.daimler-tss.com/en/imprint/> for information on the provider.