// SPDX-License-Identifier: MIT

#![feature(async_await, await_macro)]

use vehicle_information_service_client::*;

#[runtime::main]
async fn main() -> Result<(), VISClientError> {
    let client = VISClient::connect("ws://127.0.0.1:14430").await?;
    let interval: u32 = client.get("Private.Example.Interval".into()).await?;
    println!("Interval: {}", interval);
    Ok(())
}
