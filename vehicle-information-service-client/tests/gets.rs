// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await)]

use runtime::native::Native;
use vehicle_information_service_client::*;

#[runtime::test(Native)]
async fn receive_get_async() -> Result<(), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let interval: u32 = await!(client.get("Private.Example.Interval".into()))?;
    assert!(interval > 0);

    Ok(())
}
