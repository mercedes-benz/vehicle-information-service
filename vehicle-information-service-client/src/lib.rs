// SPDX-License-Identifier: MIT

use futures::prelude::*;
use log::{debug, error, warn};
use serde::de::DeserializeOwned;
use serde_json;
use std::convert::Into;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};
use vehicle_information_service::api_type::*;

pub use vehicle_information_service::api_error::ActionErrorResponse;
pub use vehicle_information_service::api_type::{ActionPath, ReqID, SubscriptionID};

#[derive(Debug)]
pub enum VISClientError {
    WebsocketError(tokio_tungstenite::tungstenite::Error),
    SerdeError(serde_json::Error),
    IoError(io::Error),
    UrlParseError(url::ParseError),
    VisError(ActionErrorResponse),
    Other,
}

impl From<tokio_tungstenite::tungstenite::Error> for VISClientError {
    fn from(websocket_error: tokio_tungstenite::tungstenite::Error) -> Self {
        VISClientError::WebsocketError(websocket_error)
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

impl From<ActionErrorResponse> for VISClientError {
    fn from(action_error: ActionErrorResponse) -> Self {
        VISClientError::VisError(action_error)
    }
}

type Result<T> = core::result::Result<T, VISClientError>;

pub struct VISClient {
    #[allow(dead_code)]
    server_address: String,
    websocket_stream: WebSocketStream<TcpStream>,
    // client: websocket::client::r#async::Client<TcpStream>,
}

impl VISClient {
    #[allow(clippy::needless_lifetimes)] // Clippy false positive
    pub async fn connect(server_address: &str) -> Result<Self> {
        let (websocket_stream, _) = connect_async(server_address).await?;
        debug!("Connected to: {}", server_address);
        Ok(Self {
            server_address: server_address.to_string(),
            websocket_stream,
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

        let (mut sink, stream) = self.websocket_stream.split();

        sink.send(Message::Text(get_msg)).await?;

        let get_stream = stream
            .map_err(Into::<VISClientError>::into)
            // Filter Websocket text messages
            .try_filter_map(|msg| {
                if let Message::Text(txt) = msg {
                    future::ok(Some(txt))
                } else {
                    future::ok(None)
                }
            })
            // Deserialize
            .and_then(|txt| {
                let txt_err = txt.clone();
                if let Ok(value) = serde_json::from_str::<ActionSuccessResponse>(&txt) {
                    return future::ok(value);
                }

                // Attempt to deserialize a VIS error
                let vis_error: std::result::Result<serde_json::Value, _> =
                    serde_json::from_str(&txt_err);
                // Workaround for https://github.com/serde-rs/json/issues/505
                // once this is fixed it should not be necessary to deserialize to Value first and then
                // to the actual type
                match vis_error {
                    Err(serde_error) => {
                        error!("{}", serde_error);
                        future::err(serde_error.into())
                    }
                    Ok(vis_error) => {
                        let vis_error = serde_json::from_value::<ActionErrorResponse>(vis_error);
                        match vis_error {
                            Err(serde_error) => {
                                error!("{}", serde_error);
                                future::err(serde_error.into())
                            }
                            Ok(vis_error) => future::err(VISClientError::VisError(vis_error)),
                        }
                    }
                }
            })
            // Filter get responses
            .try_filter_map(|response| {
                match response {
                    ActionSuccessResponse::Get {
                        request_id: resp_request_id,
                        value,
                        ..
                    } => future::ok(Some((resp_request_id, value))),
                    // No get response
                    _ => future::ok(None),
                }
            })
            // Filter get responses that have correct request_id
            .try_filter_map(|(resp_request_id, value)| {
                if request_id != resp_request_id {
                    return future::ok(None);
                }

                future::ok(Some(value))
            })
            // Deserialize value of get response
            .and_then(|value| future::ready(serde_json::from_value(value).map_err(Into::into)))
            .into_future();

        let (get_response, _stream) = get_stream.await;
        get_response.unwrap().map_err(Into::into)
    }

    /// Subscribe to the given path's vehicle signals.
    /// This will return a stream containing all incoming values
    pub async fn subscribe_raw(
        self,
        path: ActionPath,
        filters: Option<Filters>,
    ) -> Result<impl TryStream<Ok = ActionSuccessResponse, Error = VISClientError>> {
        let request_id = ReqID::default();
        let subscribe = Action::Subscribe {
            path,
            filters,
            request_id,
        };

        let subscribe_msg = serde_json::to_string(&subscribe)?;

        let (mut sink, stream) = self.websocket_stream.split();

        sink.send(Message::Text(subscribe_msg)).await?;

        Ok(stream.map_err(Into::into).try_filter_map(|msg| {
            debug!("VIS Message {:#?}", msg);
            if let Message::Text(txt) = msg {
                match serde_json::from_str::<ActionSuccessResponse>(&txt) {
                    Ok(success_response) => future::ok(Some(success_response)),
                    // propagate deserialize error to stream
                    Err(serde_error) => future::err(serde_error.into()),
                }
            } else {
                future::ok(None)
            }
        }))
    }

    /// Subscribe to the given path's vehicle signals.
    pub async fn subscribe<T>(
        self,
        path: ActionPath,
        filters: Option<Filters>,
    ) -> Result<impl TryStream<Ok = (SubscriptionID, T), Error = VISClientError>>
    where
        T: DeserializeOwned,
    {
        let (mut sink, stream) = self.websocket_stream.split();

        let request_id = ReqID::default();
        let subscribe = Action::Subscribe {
            path,
            filters,
            request_id,
        };

        let subscribe_msg = serde_json::to_string(&subscribe)?;

        // Send subscribe request to server
        sink.send(Message::Text(subscribe_msg)).await?;

        let subscription_id: Arc<Mutex<Option<SubscriptionID>>> = Default::default();

        Ok(stream
            .map_err::<VISClientError, _>(Into::into)
            .try_filter_map(move |msg| {
                debug!("VIS Message {:#?}", msg);

                if let Message::Text(txt) = msg {
                    match serde_json::from_str::<ActionSuccessResponse>(&txt) {
                        Ok(ActionSuccessResponse::Subscribe {
                            subscription_id: resp_subscription_id,
                            request_id: resp_request_id,
                            ..
                        }) => {
                            // Make sure this is actually the response to our subscription request
                            if resp_request_id != request_id {
                                return future::ok(None);
                            }
                            // Store subscription_id to make sure the stream only returns values based on this subscription
                            *subscription_id.lock().unwrap() = Some(resp_subscription_id);
                            future::ok(None)
                        }
                        Ok(ActionSuccessResponse::Subscription {
                            subscription_id: resp_subscription_id,
                            value,
                            ..
                        }) => {
                            if *subscription_id.lock().unwrap() != Some(resp_subscription_id) {
                                return future::ok(None);
                            }

                            match serde_json::from_value::<T>(value) {
                                Ok(stream_value) => {
                                    future::ok(Some((resp_subscription_id, stream_value)))
                                }
                                // propagate deserialize error to stream
                                Err(serde_error) => future::err(serde_error.into()),
                            }
                        }
                        Ok(_) => future::ok(None),
                        // propagate deserialize error to stream
                        Err(serde_error) => future::err(serde_error.into()),
                    }
                } else {
                    future::ok(None)
                }
            })
            .map_err(Into::into))
    }

    /// Subscribe to the given path's vehicle signals.
    pub async fn unsubscribe_all<T>(self) -> Result<impl Stream<Item = Result<()>>>
    where
        T: DeserializeOwned,
    {
        let request_id = ReqID::default();
        let unsubscribe_all = Action::UnsubscribeAll { request_id };

        let unsubscribe_all_msg = serde_json::to_string(&unsubscribe_all)?;

        let (mut sink, stream) = self.websocket_stream.split();

        sink.send(Message::Text(unsubscribe_all_msg)).await?;

        Ok(stream
            .map_err::<VISClientError, _>(Into::into)
            .try_filter_map(move |msg| {
                debug!("VIS Message {:#?}", msg);

                if let Message::Text(txt) = msg {
                    let action_success = serde_json::from_str::<ActionSuccessResponse>(&txt);

                    match action_success {
                        Ok(ActionSuccessResponse::UnsubscribeAll {
                            request_id: resp_request_id,
                            ..
                        }) => {
                            // Request id mismatch
                            if resp_request_id != request_id {
                                return future::ok(None);
                            }

                            future::ok(Some(()))
                        }
                        Ok(_) => future::ok(None),
                        Err(serde_error) => {
                            warn!(
                                "Failed to deserialize stream response, error: {}",
                                serde_error
                            );
                            future::ok(None)
                        }
                    }
                } else {
                    future::ok(None)
                }
            })
            .map_err(Into::into))
    }
}
