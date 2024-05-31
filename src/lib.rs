//!High level bindings to the lib nng
//!
//!Version corresponds to C library
//!
//!## Features
//!
//!- `http` - Enables http transport
//!- `tls` - Enables TLS transport
//!- `websocket` - Enables websocket transport. Implies `http` feature.
//!- `log` - Enables logging via [log](https://crates.io/crates/log) crate
//!
//!## Usage
//!
//!Basic example of client and server communication
//!
//!```rust
//!use nng_c::{options, Socket, Message, ErrorCode};
//!
//!use core::time;
//!
//!//Feel free to append zero char to avoid unnecessary allocations
//!const ADDR: &str = "ipc://nng-c-example\0";
//!const REQ_TIMEOUT: options::Req = options::Req {
//!     resend_time: Some(time::Duration::from_millis(50)),
//!     resend_tick: Some(time::Duration::from_millis(1)),
//!};
//!
//!fn server() -> Result<(), ErrorCode> {
//!    let server = Socket::rep0()?;
//!    server.listen(ADDR.into()).expect("listen");
//!
//!    loop {
//!        let msg = server.recv_msg()?;
//!        let body = msg.body();
//!        let msg = core::str::from_utf8(body).expect("utf-8 bytes");
//!        match msg {
//!            "quit" => break Ok(()),
//!            other => {
//!                println!("Received bytes(len={})={:?}", other.len(), other);
//!            }
//!        }
//!    }
//!}
//!
//!let server = std::thread::spawn(server);
//!
//!//Wait for thread to spin
//!std::thread::sleep(time::Duration::from_millis(10));
//!
//!let client = Socket::req0().expect("Create client");
//!client.set_opt(REQ_TIMEOUT).expect("Set options");
//!
//!client.connect(ADDR.into()).expect("connect");
//!
//!let mut msg = Message::new().expect("create message");
//!msg.append("ping".as_bytes()).expect("Input bytes");
//!client.send_msg(msg).expect("send message");
//!
//!let mut msg = Message::new().expect("create message");
//!msg.append("quit".as_bytes()).expect("Input bytes");
//!client.send_msg(msg).expect("send quit");
//!
//!server.join().expect("Finish server successfully");
//!```

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
