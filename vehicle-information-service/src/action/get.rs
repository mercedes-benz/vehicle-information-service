// SPDX-License-Identifier: MIT

//!
//! Retrieve the current state of a signal and respond to the requesting client.
//!

use actix::prelude::*;

use crate::action::ClientMessage;
use crate::api_error::{ActionErrorResponse, NOT_FOUND_INVALID_PATH};
use crate::api_type::{ActionPath, ActionSuccessResponse, ReqID};
use crate::signal_manager::SignalManager;
use crate::unix_timestamp_ms;

///
///[Get](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-getrequest)
///
#[derive(Debug)]
pub struct Get {
    pub path: ActionPath,
    pub request_id: ReqID,
}

impl Message for ClientMessage<Get> {
    type Result = ();
}

impl Handler<ClientMessage<Get>> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage<Get>, _ctx: &mut Self::Context) {
        if let Some(signal) = self.signal_cache.get(&msg.message.path) {
            msg.client_addr.do_send(ActionSuccessResponse::Get {
                request_id: msg.message.request_id,
                value: signal.clone(),
                timestamp: unix_timestamp_ms(),
            });
        } else {
            msg.client_addr.do_send(ActionErrorResponse::Get {
                request_id: msg.message.request_id,
                timestamp: unix_timestamp_ms(),
                error: NOT_FOUND_INVALID_PATH.into(),
            });
        }
    }
}
