// SPDX-License-Identifier: MIT

#![feature(async_await)]

use runtime::native::Native;
use vehicle_information_service_client::*;

#[runtime::test(Native)]
async fn receive_get_async() -> Result<(), VISClientError> {
    let client = VISClient::connect("ws://127.0.0.1:14430").await?;
    let interval: u32 = client.get("Private.Example.Interval".into()).await?;
    assert!(interval > 0);

    Ok(())
}
