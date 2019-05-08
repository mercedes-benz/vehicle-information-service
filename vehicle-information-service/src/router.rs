// SPDX-License-Identifier: MIT

use actix::prelude::*;
use actix_web::{middleware, ws, App};
use futures::prelude::*;
use http::status::StatusCode;
use serde_json::{from_str, json, to_string};
use uuid::Uuid;

use crate::action;
use crate::api_error::*;
use crate::api_type::*;
use crate::serialize_result;
use crate::signal_manager::{SignalManager, UpdateSignal};

pub struct ClientSession {
    /// Each client is assigned a unique identifier after connecting.
    /// This identifier can be used to identify the client in the logs.
    client_connection_id: ClientConnectionId,

    signal_manager_addr: Addr<SignalManager>,
}

impl ClientSession {
    pub fn new(signal_manager_addr: Addr<SignalManager>) -> Self {
        Self {
            client_connection_id: Uuid::new_v4(),
            signal_manager_addr,
        }
    }
}

impl Actor for ClientSession {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Client {} started", self.client_connection_id);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Cleanup client subscriptions
        self.signal_manager_addr.do_send(action::ClientMessage {
            client_connection_id: self.client_connection_id,
            client_addr: ctx.address(),
            message: action::UnsubscribeAll { request_id: None },
        });

        info!("Client {} stopped", self.client_connection_id);
    }
}

impl Message for ActionSuccessResponse {
    type Result = ();
}

impl Message for ActionErrorResponse {
    type Result = ();
}

impl Handler<ActionSuccessResponse> for ClientSession {
    type Result = ();
    fn handle(&mut self, msg: ActionSuccessResponse, ctx: &mut Self::Context) {
        // TODO replace subscribe error with subscription error
        let serialized = serialize_result(&Ok(msg), || {
            new_subscribe_error(ReqID::ReqIDInt(0), StatusCode::INTERNAL_SERVER_ERROR.into())
        });
        ctx.text(serialized)
    }
}

impl Handler<ActionErrorResponse> for ClientSession {
    type Result = ();
    fn handle(&mut self, msg: ActionErrorResponse, ctx: &mut Self::Context) {
        // TODO replace subscribe error with subscription error
        let serialized = serialize_result(&Err(msg), || {
            new_subscribe_error(ReqID::ReqIDInt(0), StatusCode::INTERNAL_SERVER_ERROR.into())
        });
        ctx.text(serialized)
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ClientSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        debug!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                debug!("Responding to `Ping` message: {} with Pong", msg);
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {}
            ws::Message::Binary(_bin) => {
                warn!("Binary message payload. This message will be ignored.");
            }
            ws::Message::Text(ref txt) => {
                // deserialize and dispatch VIS action

                match from_str::<Action>(txt) {
                    Err(e) => {
                        warn!("Deserialization error {}", e);
                        let err = new_deserialization_error();
                        if let Ok(serialized) = to_string(&err) {
                            ctx.text(serialized);
                        }
                    }
                    Ok(action) => {
                        debug!(
                            "Received action {:?} for client connection_id {}",
                            action, self.client_connection_id
                        );
                        match action {
                            Action::Subscribe {
                                path,
                                request_id,
                                filters,
                            } => {
                                self.signal_manager_addr.do_send(action::ClientMessage {
                                    client_connection_id: self.client_connection_id,
                                    client_addr: ctx.address(),
                                    message: action::Subscribe {
                                        path,
                                        request_id,
                                        filters,
                                    },
                                });
                            }
                            Action::Unsubscribe {
                                request_id,
                                subscription_id,
                            } => {
                                self.signal_manager_addr.do_send(action::ClientMessage {
                                    client_connection_id: self.client_connection_id,
                                    client_addr: ctx.address(),
                                    message: action::Unsubscribe {
                                        request_id,
                                        subscription_id,
                                    },
                                });
                            }
                            Action::Get { path, request_id } => {
                                self.signal_manager_addr.do_send(action::ClientMessage {
                                    client_connection_id: self.client_connection_id,
                                    client_addr: ctx.address(),
                                    message: action::Get { request_id, path },
                                });
                            }
                            Action::UnsubscribeAll { request_id } => {
                                self.signal_manager_addr.do_send(action::ClientMessage {
                                    client_connection_id: self.client_connection_id,
                                    client_addr: ctx.address(),
                                    message: action::UnsubscribeAll {
                                        request_id: Some(request_id),
                                    },
                                });
                            }
                            Action::Set {
                                request_id,
                                path,
                                value,
                            } => {
                                self.signal_manager_addr.do_send(action::ClientMessage {
                                    client_connection_id: self.client_connection_id,
                                    client_addr: ctx.address(),
                                    message: action::Set {
                                        request_id,
                                        path,
                                        value,
                                    },
                                });
                            }
                            // TODO implement
                            Action::Authorize { request_id, .. } => {
                                if let Ok(serialized) = to_string(&new_authorize_error(
                                    request_id,
                                    StatusCode::NOT_IMPLEMENTED.into(),
                                )) {
                                    ctx.text(serialized)
                                }
                            }
                            // TODO implement
                            Action::GetMetadata { request_id, .. } => {
                                if let Ok(serialized) = to_string(&new_get_metadata_error(
                                    request_id,
                                    StatusCode::NOT_IMPLEMENTED.into(),
                                )) {
                                    ctx.text(serialized)
                                }
                            }
                        }
                    }
                };
            }
            ws::Message::Close(close_reason) => {
                info!(
                    "Client {} sent Close message, reason: {:?}",
                    self.client_connection_id, close_reason
                );
                ctx.stop();
            }
        }
    }
}

pub struct AppState {
    signal_manager_addr: Addr<SignalManager>,
}

impl AppState {
    pub fn signal_manager_addr(&self) -> Addr<SignalManager> {
        self.signal_manager_addr.clone()
    }

    /// Set the path to the given value.
    pub fn set_signal<T>(&self, path: ActionPath, value: T)
    where
        T: serde::ser::Serialize,
    {
        self.signal_manager_addr.do_send(UpdateSignal {
            path,
            value: json!(value),
        });
    }

    /// Register a `set` action recipient. This recipient will receive all `set` action requests for all clients.
    pub fn add_set_recipient(&self, path: ActionPath, recipient: Recipient<action::Set>) {
        self.signal_manager_addr
            .do_send(action::AddSetRecipient { path, recipient });
    }

    /// Spawn a new signal stream source. A signal stream will provide signal updates for the given path.
    pub fn spawn_stream_signal_source<St>(&self, path: ActionPath, s: St)
    where
        St: TryStream + Unpin,
        St: 'static,
        St::Ok: serde::Serialize,
        St::Error: std::fmt::Debug,
    {
        let signal_manager_addr = self.signal_manager_addr.clone();

        let stream_signal_source = s.try_for_each(move |item| {
            let update = UpdateSignal {
                path: ActionPath(path.to_string()),
                value: json!(item),
            };
            signal_manager_addr.do_send(update);

            futures::future::ready(Ok(()))
        });

        actix::spawn(
            stream_signal_source
                .map_err(|e| warn!("Signal source stream error: {:?}", e))
                .compat(),
        );
    }
}

pub struct Router {}

fn ws_index(
    r: &actix_web::HttpRequest<AppState>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let addr = r.state().signal_manager_addr.clone();
    ws::start(r, ClientSession::new(addr))
}

impl Router {
    /// Create a new instance of a Router
    pub fn start() -> App<AppState> {
        let app_state = AppState {
            signal_manager_addr: SignalManager::start_default(),
        };

        // bind to the server
        App::with_state(app_state)
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(http::Method::GET).f(ws_index))
    }
}
