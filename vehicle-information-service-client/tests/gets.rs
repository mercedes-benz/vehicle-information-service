// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await, futures_api)]

use futures::future::{FutureExt, TryFutureExt};
use tokio::runtime::Runtime;
use vehicle_information_service_client::*;

#[test]
fn receive_get() {
    let mut rt = Runtime::new().unwrap();
    let f = receive_get_async().unit_error().boxed().compat();
    let interval = rt.block_on(f).unwrap().expect("Failed to receive get");

    assert!(interval > 0);
}

async fn receive_get_async() -> Result<u64, VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let interval = await!(client.get("Private.Example.Interval".into()))?;
    Ok(interval)
}
