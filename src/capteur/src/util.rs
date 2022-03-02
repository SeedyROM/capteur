//!
//! Application wide utilities.
//!

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use color_eyre::Report;

// TODO: This needs to be abstracted for all the streaming components
#[async_trait]
pub trait Consumer {
    async fn stream(&mut self) -> Result<(), Report>;
}

pub fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
