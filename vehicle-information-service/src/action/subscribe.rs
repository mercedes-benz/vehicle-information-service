// SPDX-License-Identifier: MIT

use actix::prelude::*;
use uuid::Uuid;

use crate::action::ClientMessage;
use crate::api_type::*;
use crate::signal_manager::{SignalManager, Subscription};
use crate::unix_timestamp_ms;

///
/// SUBSCRIBE request
/// [Subscribe](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
///
#[derive(Debug)]
pub struct Subscribe {
    pub path: ActionPath,
    pub request_id: ReqID,
    pub filters: Option<Filters>,
}

impl Message for ClientMessage<Subscribe> {
    type Result = ();
}

impl Handler<ClientMessage<Subscribe>> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage<Subscribe>, _ctx: &mut Self::Context) {
        let subscription_id = SubscriptionID::SubscriptionIDUUID(Uuid::new_v4());
        debug!(
            "Adding subscriber with id {} to path: {}",
            subscription_id, msg.message.path
        );

        let subscription = Subscription {
            client_addr: msg.client_addr.clone(),
            path: msg.message.path.clone(),
            subscription_id,
            filters: msg.message.filters,
            latest_signal_value: None,
            last_signal_value_client: None,
            interval_handle: None,
        };

        if let Some(subscriptions) = self.addr_to_subscription_ids.get_mut(&msg.client_addr) {
            subscriptions.push(subscription_id);
        } else {
            self.addr_to_subscription_ids
                .insert(msg.client_addr.clone(), vec![subscription_id]);
        }

        let addr = subscription.start();

        self.subscription_id_to_subscription.insert(
            subscription_id,
            (addr, msg.client_addr.clone(), msg.message.path.clone()),
        );

        if let Some(subscriptions) = self.path_to_subscription_id.get_mut(&msg.message.path) {
            subscriptions.push(subscription_id);
        } else {
            self.path_to_subscription_id
                .insert(msg.message.path, vec![subscription_id]);
        }

        let response = ActionSuccessResponse::Subscribe {
            request_id: msg.message.request_id,
            subscription_id,
            timestamp: unix_timestamp_ms(),
        };

        msg.client_addr.do_send(response);
    }
}
