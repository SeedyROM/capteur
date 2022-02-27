//!
//! The command line interface to emit messages coming from the serial/UART update ticks.
//! 
//! **Supported transport layers:**
//! 
//! - [tokio-amqp](https://crates.io/crates/tokio-amqp)
//! - *TBD: Other transport layers yet TBD!*
//! 

use color_eyre::Report;

use capteur::setup_environment;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup_environment()?;

    println!("Hello, world!");

    Ok(())
}

