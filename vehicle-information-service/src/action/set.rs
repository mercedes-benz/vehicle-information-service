// SPDX-License-Identifier: MIT

//!
//! Dispatch client Set requests to a registered set receivers and register
//! new Set receivers.
//!

use crate::action::ClientMessage;
use crate::api_error::{
    ActionErrorResponse, KnownError, NOT_FOUND_INVALID_PATH, SERVICE_UNAVAILABLE,
};
use crate::api_type::ReqID;
use crate::signal_manager::SignalManager;
use actix::prelude::*;
use log::warn;
use serde_json::Value;

use crate::api_type::{ActionPath, ActionSuccessResponse};
use crate::unix_timestamp_ms;

/// SET request
/// https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-setrequest
///
#[derive(Clone, Debug)]
pub struct Set {
    pub path: ActionPath,
    pub value: Value,
    pub request_id: ReqID,
}

impl Message for Set {
    type Result = Result<(), KnownError>;
}

impl Message for ClientMessage<Set> {
    type Result = ();
}

impl Handler<ClientMessage<Set>> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage<Set>, _ctx: &mut Self::Context) {
        let recipients = self.set_recipients.clone();
        if let Some(recipient) = recipients.get(&msg.message.path) {
            let set_message = msg.message.clone();
            if let Err(e) = recipient.do_send(set_message) {
                warn!("Failed to deliver Set message to recipient: {}", e);
                msg.client_addr.do_send(ActionErrorResponse::Set {
                    request_id: msg.message.request_id,
                    timestamp: unix_timestamp_ms(),
                    error: SERVICE_UNAVAILABLE.into(),
                });
                return;
            }

            msg.client_addr.do_send(ActionSuccessResponse::Set {
                request_id: msg.message.request_id,
                timestamp: unix_timestamp_ms(),
            });
        } else {
            // No recipient for the requested path
            msg.client_addr.do_send(ActionErrorResponse::Set {
                request_id: msg.message.request_id,
                timestamp: unix_timestamp_ms(),
                error: NOT_FOUND_INVALID_PATH.into(),
            });
        }
    }
}

pub struct AddSetRecipient {
    pub path: ActionPath,
    pub recipient: Recipient<Set>,
}

impl Message for AddSetRecipient {
    type Result = ();
}

impl Handler<AddSetRecipient> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: AddSetRecipient, _ctx: &mut Self::Context) {
        self.set_recipients.insert(msg.path, msg.recipient);
    }
}
