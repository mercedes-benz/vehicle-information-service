// SPDX-License-Identifier: MIT

//!
//! Remove all client subscriptions when a client requests an UnsubscribeAll
//! or the client disconnects.
//!

use actix::prelude::*;

use crate::action::ClientMessage;
use crate::api_type::{ActionSuccessResponse, ReqID};
use crate::signal_manager::{SignalManager, StopSubscription};
use crate::unix_timestamp_ms;

///
/// [Unsubscribe](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
///
#[derive(Debug)]
pub struct UnsubscribeAll {
    pub request_id: Option<ReqID>,
}

impl Message for ClientMessage<UnsubscribeAll> {
    type Result = ();
}

impl Handler<ClientMessage<UnsubscribeAll>> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage<UnsubscribeAll>, _ctx: &mut Self::Context) {
        for subscription_id in self
            .addr_to_subscription_ids
            .get(&msg.client_addr)
            .unwrap_or(&Vec::new())
        {
            if let Some((subscription_addr, _client_session_addr, path)) =
                self.subscription_id_to_subscription.get(&subscription_id)
            {
                subscription_addr.do_send(StopSubscription {});

                if let Some(subscription_ids) = self.path_to_subscription_id.get_mut(&path) {
                    subscription_ids.retain(|sub| sub != subscription_id)
                }
                debug!(
                    "Removed subscription with id {} to path: {}",
                    subscription_id, path
                );
            }
        }

        if let Some(request_id) = msg.message.request_id {
            let response = ActionSuccessResponse::UnsubscribeAll {
                request_id,
                timestamp: unix_timestamp_ms(),
            };
            msg.client_addr.do_send(response);
        }
    }
}
