// SPDX-License-Identifier: MIT

//!
//! Remove a specified client subscription when the client requests an unsubscribe.
//!
use actix::prelude::*;

use crate::action::ClientMessage;
use crate::api_error::{ActionErrorResponse, NOT_FOUND_INVALID_SUBSCRIPTION_ID};
use crate::api_type::{ActionSuccessResponse, ReqID, SubscriptionID};
use crate::signal_manager::{SignalManager, StopSubscription};
use crate::unix_timestamp_ms;

///
/// As a client unsubscribe from a subscription in order to no longer receive notifications.
/// [Unsubscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
///
#[derive(Debug)]
pub struct Unsubscribe {
    pub request_id: ReqID,
    pub subscription_id: SubscriptionID,
}

impl Message for ClientMessage<Unsubscribe> {
    type Result = ();
}

impl Handler<ClientMessage<Unsubscribe>> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage<Unsubscribe>, _ctx: &mut Self::Context) {
        // Make sure this subscription actually belongs to the client
        let empty = Vec::new();
        let addr_subscriptions = self
            .addr_to_subscription_ids
            .get(&msg.client_addr)
            .unwrap_or(&empty);

        if !addr_subscriptions.contains(&msg.message.subscription_id) {
            warn!(
                "Client attempted to remove subscription {} not belonging to client",
                msg.message.subscription_id
            );
            msg.client_addr.do_send(ActionErrorResponse::Unsubscribe {
                request_id: msg.message.request_id,
                subscription_id: msg.message.subscription_id,
                timestamp: unix_timestamp_ms(),
                error: NOT_FOUND_INVALID_SUBSCRIPTION_ID.into(),
            });
            return;
        }

        if let Some((subscription_addr, client_addr, path)) = self
            .subscription_id_to_subscription
            .remove(&msg.message.subscription_id)
        {
            subscription_addr.do_send(StopSubscription {});
            if let Some(subscriptions) = self.addr_to_subscription_ids.get_mut(&client_addr) {
                subscriptions.retain(|sub| *sub != msg.message.subscription_id)
            }

            if let Some(subscription_ids) = self.path_to_subscription_id.get_mut(&path) {
                subscription_ids.retain(|sub| *sub != msg.message.subscription_id)
            }
            debug!(
                "Removed subscriber with id {} to path: {}",
                msg.message.subscription_id, path
            );

            msg.client_addr.do_send(ActionSuccessResponse::Unsubscribe {
                request_id: msg.message.request_id,
                subscription_id: msg.message.subscription_id,
                timestamp: unix_timestamp_ms(),
            });
        }
    }
}
