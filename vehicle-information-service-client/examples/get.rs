#![feature(await_macro, async_await)]

use futures::future::{FutureExt, TryFutureExt};
use tokio::runtime::Runtime;
use vehicle_information_service_client::*;

fn main() {
    let mut rt = Runtime::new().unwrap();
    let f = print_current_interval().unit_error().boxed().compat();
    rt.block_on(f).unwrap().expect("Failed to receive get");
}

async fn print_current_interval() -> Result<(), VISClientError> {
    let client = await!(VISClient::connect("ws://127.0.0.1:14430"))?;
    let interval: u32 = await!(client.get("Private.Example.Interval".into()))?;
    println!("Interval: {}", interval);
    Ok(())
}
