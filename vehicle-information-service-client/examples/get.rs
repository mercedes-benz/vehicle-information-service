// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await)]

use vehicle_information_service_client::*;

#[runtime::main]
async fn main() -> Result<(), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let interval: u32 = await!(client.get("Private.Example.Interval".into()))?;
    println!("Interval: {}", interval);
    Ok(())
}
