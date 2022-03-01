//!
//! The Capteur platform and building blocks
//!

use color_eyre::Report;
use tracing_subscriber::EnvFilter;

pub mod messages;
pub mod transports;
pub mod util;
pub mod websockets;

///
/// Setup tracing and error handling with [`tracing`], [`tracing_subscriber`] and [`color_eyre`].
///
pub fn setup_environment() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
