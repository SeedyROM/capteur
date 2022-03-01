use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SensorReading {
    #[serde(rename = "measurement")]
    Measurement { value: f64, unit: String },
    #[serde(rename = "boolean")]
    Boolean { value: bool },
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    #[serde(rename = "reading")]
    Reading {
        timestamp: u128,
        sensors: BTreeMap<String, SensorReading>,
    },
}
