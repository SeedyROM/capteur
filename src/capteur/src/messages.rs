//!
//! Message definitions.
//!

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

///
/// Types of sensor readings, simple for now.
///
#[derive(Serialize, Deserialize)]
pub enum SensorReading {
    #[serde(rename = "measurement")]
    Measurement { value: f64, unit: String },
    #[serde(rename = "boolean")]
    Boolean { value: bool },
}

///
/// Messages that get sent around the capteur ecosystem.
///
#[derive(Serialize, Deserialize)]
pub enum Message {
    #[serde(rename = "reading")]
    Reading {
        timestamp: u128,
        sensors: BTreeMap<String, SensorReading>,
    },
}
