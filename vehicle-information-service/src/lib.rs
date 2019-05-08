// SPDX-License-Identifier: MIT

//!
//! For more examples check the Github repositories `examples/` folder.
//!
//! Example:
//! ```rust,no_run
//!
//! #[macro_use]
//! extern crate log;
//!
//! use actix::prelude::*;
//! use actix_web::server;
//! use futures::stream::Stream;
//! use futures_util::try_stream::TryStreamExt;
//! use futures_util::compat::Stream01CompatExt;
//! use std::net::{IpAddr, Ipv4Addr, SocketAddr};
//! use tokio_socketcan;
//!
//! use vehicle_information_service::{KnownError, Router, Set, SignalManager, UpdateSignal};
//!
//! const PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID: &str = "Private.Example.SocketCan.Last.Frame.Id";
//!
//! fn main() {
//!     env_logger::init();
//!
//!     let sys = actix::System::new("vis-example");
//!
//!     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 14430);
//!
//!     server::new(|| {
//!         let app = Router::start();
//!
//!         let can_id_stream = tokio_socketcan::CANSocket::open("vcan0")
//!             .expect("Failed to initialize CanSocket")
//!             .compat()
//!             .map_ok(|frame| frame.id());
//!
//!         app.state()
//!             .spawn_stream_signal_source(PATH_PRIVATE_EXAMPLE_SOCKETCAN_LAST_FRAME_ID.into(), can_id_stream);
//!         app
//!     })
//!     .bind(socket_addr)
//!     .unwrap()
//!     .start();
//!
//!     let _ = sys.run();
//! }
//!```
//!

#![deny(clippy::all)]
#![feature(async_await)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod action;
pub mod api_error;
pub mod api_type;
mod filter;
mod router;
mod signal_manager;

pub use action::set::Set;
pub use api_error::KnownError;
pub use api_type::ActionPath;
pub use router::{AppState, Router};
pub use signal_manager::{SignalManager, UpdateSignal};

use serde_json::to_string;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api_error::ActionErrorResponse;
use crate::api_type::ActionSuccessResponse;

pub(crate) fn unix_timestamp() -> Option<Duration> {
    SystemTime::now().duration_since(UNIX_EPOCH).ok()
}

pub fn unix_timestamp_ms() -> u128 {
    unix_timestamp().map(|t| t.as_millis()).unwrap_or_default()
}

///
/// Serialize response type and wrap the result ready to be returned by the websocket lib
///
pub(crate) fn serialize_result<F>(
    result: &Result<ActionSuccessResponse, ActionErrorResponse>,
    internal_server_error: F,
) -> String
where
    F: FnOnce() -> ActionErrorResponse,
{
    match result {
        Ok(success) => to_string(&success).unwrap_or_else(|_| {
            let error = internal_server_error();
            to_string(&error).unwrap_or_default()
        }),
        Err(error) => to_string(&error).unwrap_or_else(|_| {
            let error = internal_server_error();
            to_string(&error).unwrap_or_default()
        }),
    }
}
