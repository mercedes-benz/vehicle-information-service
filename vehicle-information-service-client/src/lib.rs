// SPDX-License-Identifier: MIT

#![feature(await_macro, async_await)]

use futures::compat::*;
use futures::prelude::*;
use futures::StreamExt;
use log::debug;
use serde::de::DeserializeOwned;
use serde_json;
use std::convert::Into;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::prelude::{Sink, Stream};
use tokio_tcp::TcpStream;
use vehicle_information_service::api_type::*;
use websocket::{ClientBuilder, OwnedMessage, WebSocketError};

#[derive(Debug)]
pub enum VISClientError {
    WebSocketError(WebSocketError),
    SerdeError(serde_json::Error),
    IoError(io::Error),
    UrlParseError(url::ParseError),
    Other,
}

impl From<WebSocketError> for VISClientError {
    fn from(ws_error: WebSocketError) -> Self {
        VISClientError::WebSocketError(ws_error)
    }
}

impl From<serde_json::Error> for VISClientError {
    fn from(json_error: serde_json::Error) -> Self {
        VISClientError::SerdeError(json_error)
    }
}

impl From<io::Error> for VISClientError {
    fn from(io_error: io::Error) -> Self {
        VISClientError::IoError(io_error)
    }
}

impl From<url::ParseError> for VISClientError {
    fn from(url_error: url::ParseError) -> Self {
        VISClientError::UrlParseError(url_error)
    }
}

type Result<T> = core::result::Result<T, VISClientError>;

pub struct VISClient {
    #[allow(dead_code)]
    server_address: String,
    client: websocket::client::r#async::Client<TcpStream>,
}

impl VISClient {
    #[allow(clippy::needless_lifetimes)] // Clippy false positive
    pub async fn connect(server_address: &str) -> Result<Self> {
        let (client, _headers) = await!(ClientBuilder::new(server_address)?
            .async_connect_insecure()
            .compat())?;
        debug!("Connected to: {}", server_address);
        Ok(Self {
            server_address: server_address.to_string(),
            client,
        })
    }

    /// Retrieve vehicle signals.
    pub async fn get<T>(self, path: ActionPath) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request_id = ReqID::default();
        let get = Action::Get { path, request_id };

        let get_msg = serde_json::to_string(&get)?;

        let (sink, stream) = self.client.split();

        await!(sink.send(OwnedMessage::Text(get_msg)).compat())?;

        let get_stream = stream
            .compat()
            .map_err(Into::<VISClientError>::into)
            // Filter Websocket text messages
            .try_filter_map(|msg| {
                if let OwnedMessage::Text(txt) = msg {
                    future::ready(Ok(Some(txt)))
                } else {
                    future::ready(Ok(None))
                }
            })
            // Deserialize
            .and_then(|txt| {
                future::ready(
                    serde_json::from_str::<ActionSuccessResponse>(&txt).map_err(Into::into),
                )
            })
            // Filter get responses
            .try_filter_map(|response| {
                match response {
                    ActionSuccessResponse::Get {
                        request_id: resp_request_id,
                        value,
                        ..
                    } => future::ready(Ok(Some((resp_request_id, value)))),
                    // No get response
                    _ => future::ready(Ok(None)),
                }
            })
            // Filter get responses that have correct request_id
            .try_filter_map(|(resp_request_id, value)| {
                if request_id != resp_request_id {
                    return future::ready(Ok(None));
                }

                future::ready(Ok(Some(value)))
            })
            // Deserialize value of get response
            .and_then(|value| future::ready(serde_json::from_value(value).map_err(Into::into)))
            .into_future();

        let (get_response, _stream) = await!(get_stream);
        get_response.unwrap().map_err(Into::into)
    }

    /// Subscribe to the given path's vehicle signals.
    /// This will return a stream containing all incoming values
    pub async fn subscribe_raw(
        self,
        path: ActionPath,
        filters: Option<Filters>,
    ) -> impl Stream<Item = ActionSuccessResponse, Error = VISClientError> {
        let request_id = ReqID::default();
        let subscribe = Action::Subscribe {
            path,
            filters,
            request_id,
        };

        let subscribe_msg =
            serde_json::to_string(&subscribe).expect("Failed to serialize subscribe");

        let (sink, stream) = self.client.split();

        await!(sink.send(OwnedMessage::Text(subscribe_msg)).compat())
            .expect("Failed to send message");
        stream
            .filter_map(|msg| {
                debug!("VIS Message {:#?}", msg);
                if let OwnedMessage::Text(txt) = msg {
                    Some(
                        serde_json::from_str::<ActionSuccessResponse>(&txt)
                            .expect("Failed to deserialize VIS response"),
                    )
                } else {
                    None
                }
            })
            .map_err(Into::into)
    }

    /// Subscribe to the given path's vehicle signals.
    pub async fn subscribe<T>(
        self,
        path: ActionPath,
        filters: Option<Filters>,
    ) -> impl Stream<Item = (SubscriptionID, T), Error = VISClientError>
    where
        T: DeserializeOwned,
    {
        let (sink, stream) = self.client.split();

        let request_id = ReqID::default();
        let subscribe = Action::Subscribe {
            path,
            filters,
            request_id,
        };

        let subscribe_msg = serde_json::to_string(&subscribe).expect("Failed to serialize message");

        await!(sink.send(OwnedMessage::Text(subscribe_msg)).compat())
            .expect("Failed to send message");

        let subscription_id: Arc<Mutex<Option<SubscriptionID>>> = Default::default();

        stream
            .filter_map(move |msg| {
                debug!("VIS Message {:#?}", msg);

                if let OwnedMessage::Text(txt) = msg {
                    let action_success = serde_json::from_str::<ActionSuccessResponse>(&txt)
                        .expect("Failed to deserialize VIS response");

                    match action_success {
                        ActionSuccessResponse::Subscribe {
                            subscription_id: resp_subscription_id,
                            request_id: resp_request_id,
                            ..
                        } => {
                            // Make sure this is actually the response to our subscription request
                            if resp_request_id != request_id {
                                return None;
                            }
                            // Store subscription_id to make sure the stream only returns values based on this subscription
                            *subscription_id.lock().unwrap() = Some(resp_subscription_id);
                            return None;
                        }
                        ActionSuccessResponse::Subscription {
                            subscription_id: resp_subscription_id,
                            value,
                            ..
                        } => {
                            if *subscription_id.lock().unwrap() != Some(resp_subscription_id) {
                                return None;
                            }

                            let stream_value = serde_json::from_value::<T>(value)
                                .expect("Failed to deserialize subscription value");
                            return Some((resp_subscription_id, stream_value));
                        }
                        _ => (),
                    }
                }
                None
            })
            .map_err(Into::into)
    }

    /// Subscribe to the given path's vehicle signals.
    pub async fn unsubscribe_all<T>(self) -> impl Stream<Item = (), Error = VISClientError>
    where
        T: DeserializeOwned,
    {
        let request_id = ReqID::default();
        let unsubscribe_all = Action::UnsubscribeAll { request_id };

        let unsubscribe_all_msg =
            serde_json::to_string(&unsubscribe_all).expect("Failed to serialize message");

        let (sink, stream) = self.client.split();

        await!(sink.send(OwnedMessage::Text(unsubscribe_all_msg)).compat())
            .expect("Failed to send message");

        stream
            .filter_map(move |msg| {
                debug!("VIS Message {:#?}", msg);

                if let OwnedMessage::Text(txt) = msg {
                    let action_success = serde_json::from_str::<ActionSuccessResponse>(&txt)
                        .expect("Failed to deserialize VIS response");
                    if let ActionSuccessResponse::UnsubscribeAll {
                        request_id: resp_request_id,
                        ..
                    } = action_success
                    {
                        if resp_request_id != request_id {
                            return None;
                        }

                        return Some(());
                    }
                    None
                } else {
                    None
                }
            })
            .map_err(Into::into)
    }
}
