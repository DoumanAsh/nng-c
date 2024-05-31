//!High level bindings to the lib nng

#![no_std]
#![warn(missing_docs)]
//Imagine enabling this shit by default
#![allow(clippy::deprecated_clippy_cfg_attr)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

extern crate alloc;

mod defs;
mod aio;
pub mod str;
pub use nng_c_sys as sys;
mod msg;
pub use msg::Message;
mod error;
pub use error::{ErrorCode, NngError};
pub mod options;
pub mod socket;
pub use socket::Socket;
pub mod tls;
pub mod utils;
