// SPDX-License-Identifier: MIT

//!
//! Errors that are returned by the WS server.
//! Includes error that are specified in the VIS specifcation [errors](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#errors).
//!
use http::status::StatusCode;
use std::fmt;
use std::io;

use crate::api_type::{ReqID, SubscriptionID};
use crate::unix_timestamp_ms;

#[cfg(test)]
mod tests {
    use crate::api_type::*;
    use crate::api_error::*;

    use http::status::StatusCode;
    use serde_json;

    #[test]
    fn serialize_deserialize_action_error() {
        let action_error: ActionError = StatusCode::INTERNAL_SERVER_ERROR.into();

        let s_action_error = serde_json::to_string(&action_error).unwrap();
        println!("{}", s_action_error);
        let d_action_error = serde_json::from_str(&s_action_error).unwrap();
        assert_eq!(action_error, d_action_error);
    }

    #[test]
    #[allow_fail]
    /// Allowed to fail until https://github.com/serde-rs/json/issues/505 is fixed
    fn serialize_deserialize_error_response() {
        let get_error_response = ActionErrorResponse::Get {
            request_id: Default::default(),
            error: StatusCode::INTERNAL_SERVER_ERROR.into(),
            timestamp: std::u128::MAX,
        };

        let s_get_error_response = serde_json::to_string(&get_error_response).unwrap();
        println!("{}", s_get_error_response);
        let d_get_error_response: serde_json::Value = serde_json::from_str(&s_get_error_response).expect("From STR");
        let d_get_error_response: ActionErrorResponse = serde_json::from_value(d_get_error_response).expect("From Value");
        assert_eq!(get_error_response, d_get_error_response);
    }

    #[test]
    fn serialize_deserialize_error_response_without_timestamp() {
        let request_id = Default::default();
        let error: ActionError = StatusCode::INTERNAL_SERVER_ERROR.into();

        let get_error_response = ActionErrorResponse::Get {
            request_id,
            error: error.clone(),
            timestamp: std::u128::MAX,
        };

        let s_get_error_response = serde_json::to_string(&get_error_response).unwrap();
        println!("{}", s_get_error_response);
        let d_get_error_response: serde_json::Value = serde_json::from_str(&s_get_error_response).expect("From STR");
        let d_get_error_response: ActionErrorResponse = serde_json::from_value(d_get_error_response).expect("From Value");
        if let ActionErrorResponse::Get { request_id: recv_request_id, error: recv_error, timestamp: _timestamp} = d_get_error_response {
            assert_eq!(request_id, recv_request_id);
            assert_eq!(error, recv_error);
        } else {
            panic!("Unexpected error response {}", d_get_error_response);
        }
    }
}

///
/// If there is an error with any of the client’s requests,
/// the server responds with an error number, reason and message.
/// [Errors Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#errors)
///
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct ActionError {
    ///
    /// HTTP Status Code Number.
    ///
    #[serde(rename = "number")]
    pub number: u16,
    // Pre-defined string value that can be used to distinguish between errors that have the same code.
    /// e.g. user_token_expired, user_token_invalid
    ///
    #[serde(rename = "reason")]
    pub reason: String,
    ///
    /// Message text describing the cause in more detail.
    /// e.g. User token has expired.
    ///
    #[serde(rename = "message")]
    pub message: String,
}

unsafe impl Send for ActionError {}
unsafe impl Sync for ActionError {}

impl ActionError {
    pub fn new(http_status_code: StatusCode, message: &'static str) -> Self {
        Self {
            number: http_status_code.as_u16(),
            reason: http_status_code
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            message: message.to_string(),
        }
    }
}

impl From<io::Error> for ActionError {
    fn from(error: io::Error) -> Self {
        warn!("io::Error {:?}", error);
        Self {
            number: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            reason: StatusCode::INTERNAL_SERVER_ERROR
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            message: String::new(),
        }
    }
}

impl From<StatusCode> for ActionError {
    fn from(status_code: StatusCode) -> Self {
        Self {
            number: status_code.as_u16(),
            reason: status_code
                .canonical_reason()
                .unwrap_or_default()
                .to_string(),
            message: String::new(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "camelCase")]
pub enum ActionErrorResponse {
    /// Error response for Authorize request
    /// [Authorize Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-authorizeerrorresponse)
    ///
    Authorize {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// Error response for failed GetMetadata request
    /// [Get VSS Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-vsserrorresponse)
    ///
    GetMetadata {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// Error response for failed GET request
    /// [Get Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-getrequest)
    ///
    Get {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// Error response for failed SET request
    /// [Set Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-setrequest)
    ///
    Set {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// Error response for failed SUBSCRIBE request
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
    ///
    Subscribe {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// Error response for failed SUBSCRIBE request
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
    ///
    Subscription {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
    ///
    SubscriptionNotification {
        #[serde(rename = "error")]
        error: ActionError,
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// [Unsubscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
    ///
    Unsubscribe {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
    ///
    /// [Unsubscribe-All Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-unsubscribeallreq)
    ///
    UnsubscribeAll {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "error")]
        error: ActionError,
        /// can currently not be deserialized, serde_json arbitrary precision bug
        /// https://github.com/serde-rs/json/issues/505
        #[serde(skip_deserializing, rename = "timestamp")]
        timestamp: u128,
    },
}

impl From<io::Error> for ActionErrorResponse {
    fn from(_: io::Error) -> Self {
        let action_error = ActionError::new(StatusCode::INTERNAL_SERVER_ERROR, "");
        ActionErrorResponse::SubscriptionNotification {
            error: action_error,
            timestamp: unix_timestamp_ms(),
            subscription_id: SubscriptionID::SubscriptionIDInt(0),
        }
    }
}

impl fmt::Display for ActionErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ActionErrorResponse:{:?}", self,)
    }
}

pub fn new_get_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::Get {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_set_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::Set {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_subscribe_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::Subscribe {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_unsubscribe_error(
    request_id: ReqID,
    subscription_id: SubscriptionID,
    error: ActionError,
) -> ActionErrorResponse {
    ActionErrorResponse::Unsubscribe {
        request_id,
        subscription_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_unsubscribe_all_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::UnsubscribeAll {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_get_metadata_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::GetMetadata {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_authorize_error(request_id: ReqID, error: ActionError) -> ActionErrorResponse {
    ActionErrorResponse::Authorize {
        request_id,
        error,
        timestamp: unix_timestamp_ms(),
    }
}

pub fn new_deserialization_error() -> ActionError {
    // TODO this does not appear to be specified in spec
    StatusCode::BAD_REQUEST.into()
}

///
/// An error that is listed in the specification error table.
/// [Error Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#errors)
///
pub struct KnownError(StatusCode, &'static str, &'static str);

impl From<KnownError> for ActionError {
    fn from(known_error: KnownError) -> Self {
        Self {
            number: known_error.0.as_u16(),
            reason: known_error.1.to_string(),
            message: known_error.2.to_string(),
        }
    }
}

pub const NOT_MODIFIED: KnownError = KnownError(
    StatusCode::NOT_MODIFIED,
    "not_modified",
    "No changes have been made by the server.",
);

pub const BAD_REQUEST: KnownError = KnownError(
    StatusCode::BAD_REQUEST,
    "bad_request",
    "The server is unable to fulfill the client request because the request is malformed.",
);

pub const BAD_REQUEST_FILTER_INVALID: KnownError = KnownError(
    StatusCode::BAD_REQUEST,
    "filter_invalid",
    "Filter requested on non-primitive type.",
);

pub const UNAUTHORIZED_USER_TOKEN_EXPIRED: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "user_token_expired",
    "User token has expired.",
);

pub const UNAUTHORIZED_USER_TOKEN_INVALID: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "user_token_invalid",
    "User token is invalid.",
);

pub const UNAUTHORIZED_USER_TOKEN_MISSING: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "user_token_missing",
    "User token is missing.",
);

pub const UNAUTHORIZED_DEVICE_TOKEN_EXPIRED: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "device_token_expired",
    "Device token has expired.",
);

pub const UNAUTHORIZED_DEVICE_TOKEN_INVALID: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "device_token_invalid",
    "Device token is invalid.",
);

pub const UNAUTHORIZED_DEVICE_TOKEN_MISSING: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "device_token_missing",
    "Device token is missing.",
);

pub const UNAUTHORIZED_TOO_MANY_ATTEMPTS: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "too_many_attempts",
    "The client has failed to authenticate too many times.",
);

pub const UNAUTHORIZED_READ_ONLY: KnownError = KnownError(
    StatusCode::UNAUTHORIZED,
    "read_only",
    "The desired signal cannot be set since it is a read only signal.",
);

pub const FORBIDDEN_USER_FORBIDDEN: KnownError = KnownError(
    StatusCode::FORBIDDEN,
    "user_forbidden",
    "The user is not permitted to access the requested resource. Retrying does not help.",
);

pub const FORBIDDEN_USER_UNKNOWN: KnownError = KnownError(
    StatusCode::FORBIDDEN,
    "user_unknown",
    "The user is unknown. Retrying does not help.",
);

pub const FORBIDDEN_DEVICE_FORBIDDEN: KnownError = KnownError(
    StatusCode::FORBIDDEN,
    "device_forbidden",
    "The device is not permitted to access the requested resource. Retrying does not help.",
);

pub const FORBIDDEN_DEVICE_UNKNOWN: KnownError = KnownError(
    StatusCode::FORBIDDEN,
    "device_unknown",
    "The device is unknown. Retrying does not help.",
);

pub const NOT_FOUND_INVALID_PATH: KnownError = KnownError(
    StatusCode::NOT_FOUND,
    "invalid_path",
    "The specified data path does not exist.",
);

pub const NOT_FOUND_PRIVATE_PATH :KnownError = KnownError(StatusCode::NOT_FOUND, "private_path", "The specified data path is private and the request is not authorized to access signals on this path.");

pub const NOT_FOUND_INVALID_SUBSCRIPTION_ID: KnownError = KnownError(
    StatusCode::NOT_FOUND,
    "invalid_subscriptionId",
    "The specified subscription was not found.",
);

pub const NOT_ACCEPTABLE: KnownError = KnownError(
    StatusCode::NOT_ACCEPTABLE,
    "not_acceptable",
    "The server is unable to generate content that is acceptable to the client",
);

pub const TOO_MANY_REQUESTS: KnownError = KnownError(
    StatusCode::TOO_MANY_REQUESTS,
    "too_many_requests",
    "The client has sent the server too many requests in a given amount of time.",
);

pub const BAD_GATEWAY :KnownError = KnownError(StatusCode::BAD_GATEWAY, "bad_gateway", "The server was acting as a gateway or proxy and received an invalid response from an upstream server.");

pub const SERVICE_UNAVAILABLE :KnownError = KnownError(StatusCode:: SERVICE_UNAVAILABLE, "service_unavailable", "The server is currently unable to handle the request due to a temporary overload or scheduled maintenance (which may be alleviated after some delay).");

pub const GATEWAY_TIMEOUT :KnownError = KnownError(StatusCode::GATEWAY_TIMEOUT, "gateway_timeout", "The server did not receive a timely response from an upstream server it needed to access in order to complete the request.");
