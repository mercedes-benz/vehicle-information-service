// SPDX-License-Identifier: MIT

#![feature(async_await, await_macro)]

use futures::compat::*;
use futures::prelude::*;
use runtime::native::Native;
use vehicle_information_service::api_type::*;
use vehicle_information_service_client::*;

#[runtime::test(Native)]
async fn receive_subscribe_async() -> Result<(), VISClientError> {
    let client = VISClient::connect("ws://127.0.0.1:14430").await?;
    let mut sub_stream = client
        .subscribe_raw("Private.Example.Interval".into(), None)
        .await
        .compat();
    let subscribe = sub_stream.next().await.expect("No next value");

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
    let client = VISClient::connect("ws://127.0.0.1:14430").await?;
    let mut sub_stream = client
        .subscribe::<u32>("Private.Example.Interval".into(), None)
        .await
        .compat();
    let response = sub_stream.next().await.expect("No next value");
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
