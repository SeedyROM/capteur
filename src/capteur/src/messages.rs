use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum SensorReading {
    #[serde(rename = "measurement")]
    Measurement { value: f64, unit: String },
    #[serde(rename = "boolean")]
    Boolean(bool),
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    #[serde(rename = "reading")]
    Reading {
        timestamp: u128,
        sensors: HashMap<String, SensorReading>,
    },
}
