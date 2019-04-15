// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await, futures_api)]

use futures::compat::*;
use futures::future::{FutureExt, TryFutureExt};
use futures::prelude::*;
use tokio::runtime::Runtime;
use vehicle_information_service::api_type::*;
use vehicle_information_service_client::*;

#[test]
fn receive_subscribe() {
    let mut rt = Runtime::new().unwrap();
    let f = receive_subscribe_async().unit_error().boxed().compat();
    let (request_id, subscription_id) = rt
        .block_on(f)
        .unwrap()
        .expect("Failed to receive subscribe");

    match subscription_id {
        SubscriptionID::SubscriptionIDUUID(uuid) => assert!(!uuid.is_nil()),
        _ => panic!("Unexpected subscription id type {}", subscription_id),
    }

    match request_id {
        ReqID::ReqIDUUID(uuid) => assert!(!uuid.is_nil()),
        _ => panic!("Unexpected request id type {}", subscription_id),
    }
}

async fn receive_subscribe_async() -> Result<(ReqID, SubscriptionID), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let mut sub_stream =
        await!(client.subscribe_raw("Private.Example.Interval".into(), None)).compat();
    let subscribe = await!(sub_stream.next()).expect("No next value");

    if let Ok(ActionSuccessResponse::Subscribe {
        request_id,
        subscription_id,
        timestamp: _,
    }) = subscribe
    {
        Ok((request_id, subscription_id))
    } else {
        panic!("Unexpected Action response {:?}", subscribe)
    }
}

#[test]
fn receive_subscription_value() {
    let mut rt = Runtime::new().unwrap();
    let f = receive_subscription_async().unit_error().boxed().compat();
    let (subscription_id, interval) = rt
        .block_on(f)
        .unwrap()
        .expect("Failed to receive subscription value");
    assert!(interval > 0);
    match subscription_id {
        SubscriptionID::SubscriptionIDUUID(uuid) => assert!(!uuid.is_nil()),
        _ => panic!("Unexpected subscription id type {}", subscription_id),
    }
}

async fn receive_subscription_async() -> Result<(SubscriptionID, u32), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let mut sub_stream =
        await!(client.subscribe::<u32>("Private.Example.Interval".into(), None)).compat();
    let sub_interval = await!(sub_stream.next()).expect("No next value");
    sub_interval
}
