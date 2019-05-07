// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await)]

use futures::compat::*;
use futures::prelude::*;
use runtime::native::Native;
use vehicle_information_service::api_type::*;
use vehicle_information_service_client::*;

#[runtime::test(Native)]
async fn receive_subscribe_async() -> Result<(), VISClientError> {
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
        match subscription_id {
            SubscriptionID::SubscriptionIDUUID(uuid) => assert!(!uuid.is_nil()),
            _ => panic!("Unexpected subscription id type {}", subscription_id),
        }

        match request_id {
            ReqID::ReqIDUUID(uuid) => assert!(!uuid.is_nil()),
            _ => panic!("Unexpected request id type {}", subscription_id),
        }
    } else {
        panic!("Unexpected Action response {:?}", subscribe)
    };

    Ok(())
}

#[runtime::test(Native)]
async fn receive_subscription_async() -> Result<(), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let mut sub_stream =
        await!(client.subscribe::<u32>("Private.Example.Interval".into(), None)).compat();
    let response = await!(sub_stream.next()).expect("No next value");
    if let Ok((subscription_id, interval)) = response {
        assert!(interval > 0);
        match subscription_id {
            SubscriptionID::SubscriptionIDUUID(uuid) => assert!(!uuid.is_nil()),
            _ => panic!("Unexpected subscription id type {}", subscription_id),
        }
        Ok(())
    } else {
        panic!("Unexpected Action response {:?}", response);
    }
}
