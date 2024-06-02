# nng-c

[![Actions Status](https://github.com/DoumanAsh/nng-c/workflows/Rust/badge.svg)](https://github.com/DoumanAsh/nng-c/actions)
[![Crates.io](https://img.shields.io/crates/v/nng-c.svg)](https://crates.io/crates/nng-c)
[![Documentation](https://docs.rs/nng-c/badge.svg)](https://docs.rs/crate/nng-c/)

High level bindings to [nng](https://github.com/nanomsg/nng).

Version corresponds to C library

## Features

- `http` - Enables http transport;
- `tls` - Enables TLS transport;
- `websocket` - Enables websocket transport. Implies `http` feature;
- `log` - Enables logging via [log](https://crates.io/crates/log) crate;
- `tracing` - Enables logging via [tracing](https://crates.io/crates/tracing) crate.

## Usage

Basic example of client and server communication

```rust
use nng_c::{options, Socket, Message, ErrorCode};

use core::time;

//Feel free to append zero char to avoid unnecessary allocations
const ADDR: &str = "ipc://nng-c-example\0";
const REQ_TIMEOUT: options::Req = options::Req {
     resend_time: Some(time::Duration::from_millis(50)),
     resend_tick: Some(time::Duration::from_millis(1)),
};

fn server() -> Result<(), ErrorCode> {
    let server = Socket::rep0()?;
    server.listen(ADDR.into()).expect("listen");

    loop {
        let msg = server.recv_msg()?;
        let body = msg.body();
        let msg = core::str::from_utf8(body).expect("utf-8 bytes");
        match msg {
            "quit" => break Ok(()),
            other => {
                println!("Received bytes(len={})={:?}", other.len(), other);
            }
        }
    }
}

let server = std::thread::spawn(server);

//Wait for thread to spin
std::thread::sleep(time::Duration::from_millis(10));

let client = Socket::req0().expect("Create client");
client.set_opt(REQ_TIMEOUT).expect("Set options");

client.connect(ADDR.into()).expect("connect");

let mut msg = Message::new().expect("create message");
msg.append("ping".as_bytes()).expect("Input bytes");
client.send_msg(msg).expect("send message");

let mut msg = Message::new().expect("create message");
msg.append("quit".as_bytes()).expect("Input bytes");
client.send_msg(msg).expect("send quit");

server.join().expect("Finish server successfully");

```
