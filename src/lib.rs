#![allow(clippy::cognitive_complexity)]
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub mod commands;
pub mod mda;
pub mod smtp;
pub mod replies;
pub mod events;
pub(crate) mod config;
pub(crate) mod stream;

pub use mda::SmtpServer;
pub use events::EventHandler;
