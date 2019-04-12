// SPDX-License-Identifier: MIT

//! This module contains the action dispatcher handlers mostly
//! implemented for the `crate::SignalManager`.
//! Each action will handle a specific VIS action e.g. retrieve the current
//! state of a signal during a Get request and return the response to the client.
//! Another example is: removing all subscriptions during an UnsubscribeAll.

use actix::prelude::*;

use crate::api_type::ClientConnectionId;
use crate::router::ClientSession;

pub mod get;
pub mod set;
pub mod subscribe;
pub mod unsubscribe;
pub mod unsubscribe_all;

pub use get::Get;
pub use set::{AddSetRecipient, Set};
pub use subscribe::Subscribe;
pub use unsubscribe::Unsubscribe;
pub use unsubscribe_all::UnsubscribeAll;

/// An incoming client message.
pub struct ClientMessage<T> {
    /// Client connection identifier, mostly for logging.
    pub client_connection_id: ClientConnectionId,
    /// Client address for responding to the client.
    pub client_addr: Addr<ClientSession>,
    /// Message the client send to the server.
    pub message: T,
}
