// SPDX-License-Identifier: MIT

//!
//! This shows a simple server providing two retrievable signals (via `get` or `subscribe`),
//! as well as one settable signal.
//! For a better understanding of the VIS specification make sure you read the specification `https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html`.
//!
#[macro_use]
extern crate log;

extern crate structopt;

use actix::prelude::*;
use actix_web::server;
use futures::prelude::*;
use futures_util::compat::Stream01CompatExt;
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio_socketcan;

use vehicle_information_service::{KnownError, Router, Set, SignalManager, UpdateSignal};

const PATH_PRIVATE_EXAMPLE_PRINT_SET: &str = "Private.Example.Print.Set";
const PATH_PRIVATE_EXAMPLE_INTERVAL: &str = "Private.Example.Interval";
const PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID: &str =
    "Private.Example.SocketCan.Last.Frame.Id";

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "Vehicle Information Service Demo")]
struct Opt {
    #[structopt(
        short = "c",
        long = "can",
        default_value = "vcan0",
        help = "CAN Interface"
    )]
    can_interface: String,

    #[structopt(
        short = "p",
        long = "port",
        default_value = "14430",
        help = "Websocket Port"
    )]
    port: u16,
}

///
/// Build with `cargo run --example server --can vcan0 --port 14430`
///
/// Connect with websocket client using e.g. wscat
/// ```
/// wscat -c "localhost:14430"
/// { "action": "Subscribe", "path": "Private.Example.Interval", "requestId": 104 }
/// ```
///
fn main() {
    env_logger::init();

    let sys = actix::System::new("vis-server-example");

    let opt = Opt::from_args();

    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opt.port);

    info!("Starting server");

    server::new(move || {
        let app = Router::start();

        let interval_signal_source =
            IntervalSignalSource::new(app.state().signal_manager_addr().clone());
        interval_signal_source.start();

        // Use a [futures Stream](https://docs.rs/futures-preview/0.3.0-alpha.15/futures/prelude/trait.Stream.html) as a signal source.
        // This stream will provide data that can be retrieved via `get` and `subscribe`.
        // You must set up a vcan0 interface for this example.
        let can_id_stream = tokio_socketcan::CANSocket::open(&opt.can_interface)
            .unwrap()
            .compat()
            .map_ok(|frame| frame.id());

        app.state().spawn_stream_signal_source(
            PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID.into(),
            can_id_stream,
        );

        // A set recipient will receive `set` requests for the given path.
        // You may then handle the signal value according to the path and value.
        let example_set = PrintSetRecipient::start_default();
        app.state().add_set_recipient(
            PATH_PRIVATE_EXAMPLE_PRINT_SET.into(),
            example_set.recipient().clone(),
        );

        app
    })
    .bind(socket_addr)
    .unwrap()
    .start();

    let _ = sys.run();
}

/// This demonstrates a signal source that implements an actor
/// and updates the signal value via the `SignalManager` actor.
/// The counter value can be accessed via: `Private.Example.Interval`.
pub(crate) struct IntervalSignalSource {
    signal_manager_addr: Addr<SignalManager>,
    interval_handle: Option<SpawnHandle>,
    count: Arc<AtomicUsize>,
}

impl IntervalSignalSource {
    pub fn new(signal_manager_addr: Addr<SignalManager>) -> Self {
        IntervalSignalSource {
            signal_manager_addr,
            interval_handle: None,
            count: Default::default(),
        }
    }
}

impl Actor for IntervalSignalSource {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.interval_handle = self.interval_handle.or_else(|| {
            Some(ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
                let v = act.count.fetch_add(1, Ordering::SeqCst);

                // Update the path value, this notifies all current subscibers.
                // The new value can also be retrieved via `get`.
                let update = UpdateSignal {
                    path: PATH_PRIVATE_EXAMPLE_INTERVAL.into(),
                    value: json!(v),
                };
                act.signal_manager_addr.do_send(update);
            }))
        });
    }
}

/// This `set` recipient will handle incoming SET requests
/// and print the incoming result. Implement `Handler<Set>` in your actors
/// to deal with incoming `set` requests.
///
/// Register your `set` recipient to a specified path like this:
///
/// ```
/// use vehicle_information_service::{KnownError, Router, Set};
///
/// let app = Router::start();
/// app.state().add_set_recipient(
///     PATH_PRIVATE_EXAMPLE_PRINT_SET.into(),
///     example_set.recipient().clone(),
/// );
///```
#[derive(Default)]
struct PrintSetRecipient {}

impl Actor for PrintSetRecipient {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!(
            "Print `set`-recipient started, PATH: {}",
            PATH_PRIVATE_EXAMPLE_PRINT_SET
        );
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        info!(
            "Print `set`-recipient stopped, PATH: {}",
            PATH_PRIVATE_EXAMPLE_PRINT_SET
        );
    }
}

impl Handler<Set> for PrintSetRecipient {
    type Result = Result<(), KnownError>;

    fn handle(&mut self, msg: Set, _ctx: &mut Context<Self>) -> Result<(), KnownError> {
        info!("Received SET for path `{}`, value: {}", msg.path, msg.value);
        Ok(())
    }
}
