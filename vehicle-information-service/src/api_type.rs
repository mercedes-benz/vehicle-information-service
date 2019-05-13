// SPDX-License-Identifier: MIT

//!
//! Types as defined by the VIS specification.
//!
use crate::api_error::ActionErrorResponse;
use actix::Message;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Number, Value};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use uuid;

#[cfg(test)]
mod tests {
    use crate::api_type::*;
    use serde_json;

    #[test]
    fn action_path_equality_caseinsensite() {
        assert_eq!(
            ActionPath("path".to_string()),
            ActionPath("PATH".to_string())
        );
        assert_eq!(
            ActionPath("pAtH".to_string()),
            ActionPath("PaTh".to_string())
        );
    }

    #[test]
    fn serialize_deserialize_req_id_int() {
        let req_id_int = ReqID::ReqIDInt(100);
        let s_req_id_int = serde_json::to_string(&req_id_int).unwrap();
        let d_req_id_int = serde_json::from_str(&s_req_id_int).unwrap();
        assert_eq!(req_id_int, d_req_id_int);
    }

    #[test]
    fn serialize_deserialize_req_id_uuid() {
        let req_id_uuid = ReqID::ReqIDUUID(uuid::Uuid::new_v4());
        let s_req_id_uuid = serde_json::to_string(&req_id_uuid).unwrap();
        let d_req_id_uuid = serde_json::from_str(&s_req_id_uuid).unwrap();
        assert_eq!(req_id_uuid, d_req_id_uuid);
    }

    #[test]
    fn serialize_deserialize_subscription_id_int() {
        let sub_id_int = SubscriptionID::SubscriptionIDInt(100);
        let s_sub_id_int = serde_json::to_string(&sub_id_int).unwrap();
        let d_sub_id_int = serde_json::from_str(&s_sub_id_int).unwrap();
        assert_eq!(sub_id_int, d_sub_id_int);
    }

    #[test]
    fn serialize_deserialize_subscription_id_uuid() {
        let sub_id_uuid = SubscriptionID::SubscriptionIDUUID(uuid::Uuid::new_v4());
        let s_sub_id_uuid = serde_json::to_string(&sub_id_uuid).unwrap();
        let d_sub_id_uuid = serde_json::from_str(&s_sub_id_uuid).unwrap();
        assert_eq!(sub_id_uuid, d_sub_id_uuid);
    }
}

/// Unique id value specified by the client.
/// Returned by the server in the response and used by
/// client to link the request and response messages.
/// May be a Universally Unique Identifier (UUID)
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReqID {
    ReqIDInt(u64),
    ReqIDUUID(uuid::Uuid),
}

impl fmt::Display for ReqID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReqID::ReqIDInt(i) => write!(f, "ReqId int {}", i),
            ReqID::ReqIDUUID(u) => write!(f, "ReqId uuid {}", u),
        }
    }
}

impl Default for ReqID {
    fn default() -> Self {
        let id = uuid::Uuid::new_v4();
        ReqID::ReqIDUUID(id)
    }
}


/// Custom implementation because by spec it's not a JSON Number or JSON String(UUID)
/// but a JSON string that contains an UUID or int
impl<'de> Deserialize<'de> for ReqID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ReqIDVisitor;

        impl<'de> Visitor<'de> for ReqIDVisitor {
            type Value = ReqID;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an integer or a uuid")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Ok(uuid) = uuid::Uuid::from_str(value) {
                    Ok(ReqID::ReqIDUUID(uuid))
                } else if let Ok(number) = value.parse() {
                    Ok(ReqID::ReqIDInt(number))
                } else {
                    Err(E::custom(format!(
                        "string is not a uuid nor an integer: {}",
                        value
                    )))
                }
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Ok(uuid) = uuid::Uuid::from_str(&value) {
                    Ok(ReqID::ReqIDUUID(uuid))
                } else if let Ok(number) = value.parse() {
                    Ok(ReqID::ReqIDInt(number))
                } else {
                    Err(E::custom(format!(
                        "string is not a uuid nor an integer: {}",
                        value
                    )))
                }
            }
        }

        deserializer.deserialize_string(ReqIDVisitor)
    }
}



/// Custom implementation because by spec it's not a JSON Number or JSON String(UUID)
/// but a JSON string that contains an UUID or int
impl Serialize for ReqID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ReqID::ReqIDInt(i) => serializer.serialize_str(&i.to_string()),
            ReqID::ReqIDUUID(u) => serializer.serialize_str(&u.to_string()),
        }
    }
}

///
/// Value returned by the server to uniquely identify each subscription.
///
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum SubscriptionID {
    SubscriptionIDInt(i64),
    SubscriptionIDUUID(uuid::Uuid),
}

impl fmt::Display for SubscriptionID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SubscriptionID::SubscriptionIDInt(i) => write!(f, "SubId int {}", i),
            SubscriptionID::SubscriptionIDUUID(u) => write!(f, "SubId uuid {}", u),
        }
    }
}

impl Default for SubscriptionID {
    fn default() -> Self {
        let id = uuid::Uuid::new_v4();
        SubscriptionID::SubscriptionIDUUID(id)
    }
}

struct SubscriptionIDVisitor;

impl<'de> Visitor<'de> for SubscriptionIDVisitor {
    type Value = SubscriptionID;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a uuid")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(uuid) = uuid::Uuid::from_str(value) {
            Ok(SubscriptionID::SubscriptionIDUUID(uuid))
        } else if let Ok(number) = value.parse() {
            Ok(SubscriptionID::SubscriptionIDInt(number))
        } else {
            Err(E::custom(format!(
                "string is not a uuid nor an integer: {}",
                value
            )))
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(uuid) = uuid::Uuid::from_str(&value) {
            Ok(SubscriptionID::SubscriptionIDUUID(uuid))
        } else if let Ok(number) = value.parse() {
            Ok(SubscriptionID::SubscriptionIDInt(number))
        } else {
            Err(E::custom(format!(
                "string is not a uuid nor an integer: {}",
                value
            )))
        }
    }
}

/// Custom implementation because by spec it's not a JSON Number or JSON String(UUID)
/// but a JSON string that contains an UUID or int
impl Serialize for SubscriptionID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            SubscriptionID::SubscriptionIDInt(i) => serializer.serialize_str(&i.to_string()),
            SubscriptionID::SubscriptionIDUUID(u) => serializer.serialize_str(&u.to_string()),
        }
    }
}

/// Custom implementation because by spec it's not a JSON Number or JSON String(UUID)
/// but a JSON string that contains an UUID or int
impl<'de> Deserialize<'de> for SubscriptionID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(SubscriptionIDVisitor)
    }
}

///
/// The path to the desired vehicle signal(s), as defined by the Vehicle Signal Specification (VSS).
///
/// # Examples
/// `ActionPath("Signal.Vehicle.Speed".to_string())`
///
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActionPath(pub String);

impl ActionPath {
    /// Create a new instance out of a str
    pub fn new(path: &str) -> ActionPath {
        ActionPath(path.to_string())
    }
}

impl PartialEq for ActionPath {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

impl Eq for ActionPath {}

impl Hash for ActionPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

impl fmt::Display for ActionPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ActionPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

///
/// [Action](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#action)
///
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
    ///
    /// Enables client to pass security tokens for Security Principals to the server to support access-control.
    ///
    #[serde(alias = "authorize")]
    #[serde(alias = "Authorize")]
    Authorize,
    ///
    /// Allows the client to request metadata describing signals and data attributes that are potentially accessible.
    ///
    #[serde(alias = "getMetadata")]
    #[serde(alias = "GetMetadata")]
    GetMetadata,
    ///
    /// Enables the client to get a value once.
    ///
    #[serde(alias = "get")]
    #[serde(alias = "Get")]
    Get,
    ///
    /// Enables the client to set a value once.
    ///
    #[serde(alias = "set")]
    #[serde(alias = "Set")]
    Set,
    ///
    /// Enables the client to request notifications containing a JSON data structure with values for one or more vehicle
    /// signals and/or data attributes. The client requests that it is notified when the signal changes on the server.
    ///
    #[serde(alias = "subscribe")]
    #[serde(alias = "Subscribe")]
    Subscribe,
    ///
    /// Enables the server to send notifications to the client containing a JSON data structure with values for one or
    /// more vehicle signals and/or data attributes.
    ///
    #[serde(alias = "subscription")]
    #[serde(alias = "Subscription")]
    Subscription,
    ///
    /// Allows the client to notify the server that it should no longer receive notifications based
    /// on that subscription.
    ///
    #[serde(alias = "unsubscribe")]
    #[serde(alias = "Unsubscribe")]
    Unsubscribe,
    ///
    /// Allows the client to notify the server that it should no longer receive notifications for
    /// any active subscription.
    ///
    #[serde(alias = "unsubscribeAll")]
    #[serde(alias = "UnsubscribeAll")]
    UnsubscribeAll,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            ActionType::Authorize => "AUTHORIZE",
            ActionType::GetMetadata => "GET_METADATA",
            ActionType::Get => "GET",
            ActionType::Set => "SET",
            ActionType::Subscribe => "SUBSCRIBE",
            ActionType::Subscription => "SUBSCRIPTION",
            ActionType::Unsubscribe => "UNSUBSCRIBE",
            ActionType::UnsubscribeAll => "UNSUBSCRIBE_ALL",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilterRange {
    #[serde(default)]
    pub below: Option<Number>,
    #[serde(default)]
    pub above: Option<Number>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Filters {
    #[serde(default)]
    pub interval: Option<u64>,
    #[serde(default)]
    pub range: Option<FilterRange>,
    #[serde(default)]
    #[serde(rename = "minChange")]
    pub min_change: Option<Number>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Action {
    ///
    /// AUTHORIZE request
    /// [Authorize Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-authorizerequest)
    ///
    #[serde(alias = "authorize")]
    #[serde(alias = "Authorize")]
    Authorize {
        tokens: Value,
        #[serde(rename = "requestId")]
        request_id: ReqID,
    },
    ///
    /// Metadata request
    /// [Metadata Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-metadatarequest)
    ///
    #[serde(alias = "getMetadata")]
    #[serde(alias = "GetMetadata")]
    GetMetadata {
        path: ActionPath,
        #[serde(rename = "requestId")]
        request_id: ReqID,
    },
    ///
    /// GET request
    /// [Get Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-getrequest)
    ///
    #[serde(alias = "get")]
    #[serde(alias = "Get")]
    Get {
        path: ActionPath,
        #[serde(rename = "requestId")]
        request_id: ReqID,
    },
    ///
    /// SET request
    /// [Set Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-setrequest)
    ///
    #[serde(alias = "set")]
    #[serde(alias = "Set")]
    Set {
        path: ActionPath,
        value: Value,
        #[serde(rename = "requestId")]
        request_id: ReqID,
    },
    ///
    /// SUBSCRIBE request
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
    ///
    #[serde(alias = "subscribe")]
    #[serde(alias = "Subscribe")]
    Subscribe {
        path: ActionPath,
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        filters: Option<Filters>,
    },
    ///
    /// [Unsubscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
    ///
    #[serde(alias = "unsubscribe")]
    #[serde(alias = "Unsubscribe")]
    Unsubscribe {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
    },
    ///
    /// [Unsubscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
    ///
    #[serde(alias = "unsubscribeAll")]
    #[serde(alias = "UnsubscribeAll")]
    UnsubscribeAll {
        #[serde(rename = "requestId")]
        request_id: ReqID,
    },
}

impl Message for Action {
    type Result = Result<ActionSuccessResponse, ActionErrorResponse>;
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "camelCase")]
pub enum ActionSuccessResponse {
    ///
    /// Response for successful GET request
    /// [Get Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-getrequest)
    ///
    Get {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        value: Value,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
    ///
    /// Response for successful SET request
    /// [Set Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#dfn-setrequest)
    ///
    Set {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
    ///
    /// [Unsubscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe)
    ///
    Unsubscribe {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
    ///
    /// [Unsubscribe-All Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#unsubscribe-all)
    ///
    UnsubscribeAll {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
    ///
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#idl-def-subscriptionnotification)
    ///
    Subscription {
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
        value: Value,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
    ///
    /// Response for successful SUBSCRIBE request
    /// [Subscribe Doc](https://w3c.github.io/automotive/vehicle_data/vehicle_information_service.html#subscribe)
    ///
    Subscribe {
        #[serde(rename = "requestId")]
        request_id: ReqID,
        #[serde(rename = "subscriptionId")]
        subscription_id: SubscriptionID,
        // serde_json currently does not support deserializing u128
        #[serde(skip_deserializing)]
        timestamp: u128,
    },
}

///
/// Websocket client connection id
///
pub type ClientConnectionId = uuid::Uuid;
