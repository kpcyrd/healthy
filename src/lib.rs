#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

use std::sync::{Arc, RwLock};

pub type StatusLock = Arc<RwLock<PushStatus>>;
lazy_static! {
    pub static ref STATUS: StatusLock = Arc::new(RwLock::new(PushStatus::default()));
}

pub mod args;
pub mod config;
pub mod errors;
pub mod ping;
pub mod push;
pub mod server;
use crate::server::PushStatus;
