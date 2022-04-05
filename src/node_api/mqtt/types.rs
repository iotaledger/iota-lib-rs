// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! MQTT types

use crate::Result;

use regex::RegexSet;

use std::{collections::HashMap, sync::Arc, time::Duration};

type TopicHandler = Box<dyn Fn(&TopicEvent) + Send + Sync>;

pub(crate) type TopicHandlerMap = HashMap<Topic, Vec<Arc<TopicHandler>>>;

/// An event from a MQTT topic.

#[derive(Debug, Clone, serde::Serialize)]
pub struct TopicEvent {
    /// the MQTT topic.
    pub topic: String,
    /// The MQTT event payload.
    pub payload: String,
}

/// Mqtt events.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MqttEvent {
    /// Client was connected.
    Connected,
    /// Client was disconnected.
    Disconnected,
}

/// The MQTT broker options.

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct BrokerOptions {
    #[serde(default = "default_broker_automatic_disconnect", rename = "automaticDisconnect")]
    pub(crate) automatic_disconnect: bool,
    #[serde(default = "default_broker_timeout")]
    pub(crate) timeout: Duration,
    #[serde(default = "default_broker_use_ws", rename = "useWs")]
    pub(crate) use_ws: bool,
    #[serde(default = "default_broker_port")]
    pub(crate) port: u16,
    #[serde(default = "default_max_reconnection_attempts", rename = "maxReconnectionAttempts")]
    pub(crate) max_reconnection_attempts: usize,
}

fn default_broker_automatic_disconnect() -> bool {
    true
}

fn default_broker_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_broker_use_ws() -> bool {
    true
}

fn default_broker_port() -> u16 {
    1883
}

fn default_max_reconnection_attempts() -> usize {
    0
}

impl Default for BrokerOptions {
    fn default() -> Self {
        Self {
            automatic_disconnect: default_broker_automatic_disconnect(),
            timeout: default_broker_timeout(),
            use_ws: default_broker_use_ws(),
            port: default_broker_port(),
            max_reconnection_attempts: default_max_reconnection_attempts(),
        }
    }
}

impl BrokerOptions {
    /// Creates the default broker options.
    pub fn new() -> Self {
        Default::default()
    }

    /// Whether the MQTT broker should be automatically disconnected when all topics are unsubscribed or not.
    pub fn automatic_disconnect(mut self, automatic_disconnect: bool) -> Self {
        self.automatic_disconnect = automatic_disconnect;
        self
    }

    /// Sets the timeout used for the MQTT operations.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the use_ws used for the MQTT operations.
    pub fn use_ws(mut self, use_ws: bool) -> Self {
        self.use_ws = use_ws;
        self
    }

    /// Sets the port used for the MQTT operations.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the maximum number of reconnection attempts. 0 is unlimited.
    pub fn max_reconnection_attempts(mut self, max_reconnection_attempts: usize) -> Self {
        self.max_reconnection_attempts = max_reconnection_attempts;
        self
    }
}

/// A MQTT topic.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Topic(String);

impl TryFrom<String> for Topic {
    type Error = crate::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl Topic {
    /// Creates a new topic and checks if it's valid.
    pub fn try_new(topic: String) -> Result<Self> {
        let available_topics = lazy_static!(
        RegexSet::new(&[
            // Milestone topics
            r"milestones/latest",
            r"milestones/confirmed",
            // Message topics
            r"messages",
            r"messages/referenced",
            r"messages/transaction",
            r"messages/transaction/tagged-data",
            r"messages/transaction/tagged-data/0x([a-f0-9]{64})",
            r"messages/milestone",
            r"messages/tagged-data",
            r"messages/tagged-data/0x([a-f0-9]{64})",
            r"messages/0x([a-f0-9]{64})/metadata",
            // Transaction topics
            r"transactions/0x([a-f0-9]{64})/included-message",
            // Output topics
            r"outputs/0x([a-f0-9]{64})(\d{4})",
            r"outputs/aliases/0x([a-f0-9]{20})",
            r"outputs/nfts/0x([a-f0-9]{20})",
            r"outputs/foundries/0x([a-f0-9]{26})",
            // BIP-173 compliant bech32 address
            r"outputs/unlock/(+|address|storage-return|expiration-return|state-controller|governor|immutable-alias)/[\x21-\x7E]{1,30}1[A-Za-z0-9]+",
            // BIP-173 compliant bech32 address
            r"outputs/unlock/(+|address|storage-return|expiration-return|state-controller|governor|immutable-alias)/[\x21-\x7E]{1,30}1[A-Za-z0-9]+/spent",
        ]).expect("cannot build regex set") => RegexSet);

        if available_topics.is_match(&topic) {
            Ok(Self(topic))
        } else {
            Err(crate::Error::InvalidMqttTopic(topic))
        }
    }

    /// Creates a new topic without checking if the given string represents a valid topic.
    pub fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    /// Returns the topic.
    pub fn topic(&self) -> &str {
        &self.0
    }
}
