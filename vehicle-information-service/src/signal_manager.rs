// SPDX-License-Identifier: MIT
use actix::prelude::*;
use log::warn;
use serde_json::Value;

use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use crate::action::set::Set;
use crate::api_error::{ActionErrorResponse, BAD_REQUEST_FILTER_INVALID};
use crate::api_type::{ActionPath, ActionSuccessResponse, Filters, SubscriptionID};
use crate::filter;
use crate::router::ClientSession;
use crate::unix_timestamp_ms;

#[derive(Default)]
pub struct SignalManager {
    pub(crate) signal_cache: HashMap<ActionPath, Value>,

    pub(crate) addr_to_subscription_ids: HashMap<Addr<ClientSession>, Vec<SubscriptionID>>,
    pub(crate) path_to_subscription_id: HashMap<ActionPath, Vec<SubscriptionID>>,
    pub(crate) subscription_id_to_subscription:
        HashMap<SubscriptionID, (Addr<Subscription>, Addr<ClientSession>, ActionPath)>,

    /// Recipients that are informed on incoming `SET` actions.
    pub(crate) set_recipients: HashMap<ActionPath, Recipient<Set>>,
}

impl Actor for SignalManager {
    type Context = Context<Self>;
}

impl Supervised for SignalManager {
    fn restarting(&mut self, _ctx: &mut Context<SignalManager>) {
        warn!(
            "Signal Manager actor is restarting, current subscription count: {}",
            self.subscription_id_to_subscription.len()
        );
    }
}

#[derive(Debug, Clone)]
pub struct UpdateSignal {
    pub path: ActionPath,
    pub value: Value,
}

impl Message for UpdateSignal {
    type Result = ();
}

impl Handler<UpdateSignal> for SignalManager {
    type Result = ();

    fn handle(&mut self, msg: UpdateSignal, _ctx: &mut Self::Context) {
        let subscription_ids = self.path_to_subscription_id.get(&msg.path);

        for subscription_id in subscription_ids.unwrap_or(&Vec::new()) {
            match self
                .subscription_id_to_subscription
                .get_mut(&subscription_id)
            {
                None => warn!("Missing addr for SubscriptionId {}", subscription_id),
                Some((subscription_addr, _client_session_addr, _path)) => {
                    let notify = NotifySubscriber {
                        signal_value: msg.value.clone(),
                    };
                    subscription_addr.do_send(notify);
                }
            }
        }

        debug!("Updating signal cache value for path: {}", msg.path);
        self.signal_cache.insert(msg.path, msg.value);
    }
}

/// A client subscription.
#[derive(Clone)]
pub struct Subscription {
    /// Client who send subscribe request
    pub client_addr: Addr<ClientSession>,

    /// Signal path
    pub path: ActionPath,

    /// A random subscriptionId that is generated when creating a subscription.
    /// This is also passed when sending SubscriptionNotifications.
    pub subscription_id: SubscriptionID,

    /// Filters e.g. minChange requested by client when subscribing
    pub filters: Option<Filters>,

    /// Latest known signal value, this may not have been sent to the client yet
    /// if the filter did not match or if this an interval based subscription.
    pub latest_signal_value: Option<Value>,

    /// Last value send to client via SubscriptionNotification, contains timestamp when last value was sent
    pub last_signal_value_client: Option<(SystemTime, Value)>,

    /// Handle used when the subscription contains an interval filter
    pub interval_handle: Option<SpawnHandle>,
}

impl Subscription {
    pub fn send_client_notification(&mut self, signal_value: &Value) {
        match filter::matches(signal_value, &self.last_signal_value_client, &self.filters) {
            Ok(true) => {
                debug!(
                    "Notifiying SubscriptionId {} of value change",
                    self.subscription_id
                );

                self.last_signal_value_client = Some((SystemTime::now(), signal_value.clone()));
                let s = ActionSuccessResponse::Subscription {
                    subscription_id: self.subscription_id,
                    value: signal_value.clone(),
                    timestamp: unix_timestamp_ms(),
                };
                self.client_addr.do_send(s);
            }
            // Value is filtered and will not be send to client
            Ok(false) => {
                debug!(
                    "Update does not match filter for SubscriptionId {}",
                    self.subscription_id
                );
            }
            Err(filter::Error::ValueIsNotANumber) => {
                let s = ActionErrorResponse::SubscriptionNotification {
                    subscription_id: self.subscription_id,
                    error: BAD_REQUEST_FILTER_INVALID.into(),
                    timestamp: unix_timestamp_ms(),
                };
                self.client_addr.do_send(s);
            }
        }
    }
}

impl fmt::Display for Subscription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subscription: {}", self.subscription_id,)
    }
}

impl PartialEq<Subscription> for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.subscription_id == other.subscription_id
    }
}

#[derive(Debug, Clone)]
pub struct NotifySubscriber {
    pub signal_value: Value,
}

impl Message for NotifySubscriber {
    type Result = ();
}

impl Actor for Subscription {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!(
            "Started subscription actor, subscription_id: {}",
            self.subscription_id
        );

        if let Some(ref filters) = self.filters {
            if let Some(interval) = filters.interval {
                self.interval_handle = self.interval_handle.or_else(|| {
                    debug!("Starting subscription interval {}", interval);

                    Some(
                        ctx.run_interval(std::time::Duration::from_secs(interval), |act, _ctx| {
                            if let Some(ref latest_signal_value) = act.latest_signal_value {
                                let value = latest_signal_value.clone();
                                act.send_client_notification(&value);
                            }
                        }),
                    )
                });
            }
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!(
            "Stopped subscription actor, subscription_id: {}",
            self.subscription_id
        );
    }
}

impl Handler<NotifySubscriber> for Subscription {
    type Result = ();

    fn handle(&mut self, msg: NotifySubscriber, _ctx: &mut Self::Context) {
        self.latest_signal_value = Some(msg.signal_value.clone());

        // Interval based subscriptions are handled in the timer
        if self
            .filters
            .as_ref()
            .map(|ref x| x.interval.is_none())
            .unwrap_or(true)
        {
            debug!("{:#?}", self.filters);
            self.send_client_notification(&msg.signal_value);
        }
    }
}

pub struct StopSubscription;

impl Message for StopSubscription {
    type Result = ();
}

impl Handler<StopSubscription> for Subscription {
    type Result = ();

    fn handle(&mut self, _: StopSubscription, ctx: &mut Self::Context) {
        ctx.stop();
    }
}
