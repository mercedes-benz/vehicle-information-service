// SPDX-License-Identifier: MIT

use actix::prelude::*;

///
/// AUTHORIZE request
/// https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-authorizerequest
///
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
struct AuthorizeMessage {
    tokens: Value,
    #[serde(rename = "requestId")]
    request_id: ReqID,
}

impl Handler for Authorize {
    type Result = Result<Ok, Error>;
    fn handle(&mut self, authorize: Authorize, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
        unimplemented!();
    }
}