use nng_c::{options, Socket, Message, NngError};

use core::time;

mod rt;

#[test]
fn should_do_req_resp_inproc() {
    const ADDR: &str =  "inproc://req_resp_test\0";

    const FIRST: u64 = u64::MAX - 1;
    const SECOND: u32 = u32::MAX - 1;
    const THIRD: u16 = u16::MAX - 1;
    const BYTES: &[u8] = &[1, 10, 20, 50, 100];

    let options = options::Req {
        resend_time: Some(time::Duration::from_millis(50)),
        resend_tick: Some(time::Duration::from_millis(1)),
    };
    let client = Socket::req0().expect("Create client");
    client.set_opt(options).expect("Set options");
    let server = Socket::rep0().expect("Create server");

    server.listen(ADDR.into()).expect("listen");
    client.connect(ADDR.into()).expect("connect");

    let mut req = Message::new().expect("Create message");
    req.append(BYTES).expect("append bytes");
    req.append_u64(FIRST).expect("Apped u64");
    req.append_u32(SECOND).expect("Append u32");
    req.append_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    client.send_msg(req).expect("Send message");

    let mut resp = server.recv_msg().expect("Get message");
    let third = resp.pop_u16().expect("get u16");
    let second = resp.pop_u32().expect("get u32");
    let first = resp.pop_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
    resp.clear();

    let mut req = Message::new().expect("Create message");
    req.insert(BYTES).expect("append bytes");
    req.insert_u64(FIRST).expect("Apped u64");
    req.insert_u32(SECOND).expect("Append u32");
    req.insert_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    client.send_msg(req).expect("Send message");

    let mut resp = server.recv_msg().expect("Get message");
    let third = resp.pop_front_u16().expect("get u16");
    let second = resp.pop_front_u32().expect("get u32");
    let first = resp.pop_front_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
}

#[test]
fn should_do_req_resp_async_inproc() {
    const ADDR: &str =  "inproc://req_resp_test_async\0";

    const FIRST: u64 = u64::MAX - 1;
    const SECOND: u32 = u32::MAX - 1;
    const THIRD: u16 = u16::MAX - 1;
    const BYTES: &[u8] = &[1, 10, 20, 50, 100];

    let client = Socket::req0().expect("Create client");
    let server = Socket::rep0().expect("Create server");

    server.listen(ADDR.into()).expect("listen");
    client.connect(ADDR.into()).expect("connect");

    let mut req = Message::new().expect("Create message");
    req.append(BYTES).expect("append bytes");
    req.append_u64(FIRST).expect("Apped u64");
    req.append_u32(SECOND).expect("Append u32");
    req.append_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    let req = client.send_msg_async(req).expect("Send message");
    rt::run(req).expect("Deliver message");

    let resp = server.recv_msg_async().expect("create future");
    let mut resp = rt::run(resp).expect("Get message").expect("To have message");
    let third = resp.pop_u16().expect("get u16");
    let second = resp.pop_u32().expect("get u32");
    let first = resp.pop_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
    resp.clear();

    let mut req = Message::new().expect("Create message");
    req.insert(BYTES).expect("append bytes");
    req.insert_u64(FIRST).expect("Apped u64");
    req.insert_u32(SECOND).expect("Append u32");
    req.insert_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    client.send_msg(req).expect("Send message");

    let mut resp = server.recv_msg().expect("Get message");
    let third = resp.pop_front_u16().expect("get u16");
    let second = resp.pop_front_u32().expect("get u32");
    let first = resp.pop_front_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
}

#[test]
fn should_do_req_resp_async_tcp() {
    const ADDR: &str =  "tcp://127.0.0.1:65000\0";

    const FIRST: u64 = u64::MAX - 1;
    const SECOND: u32 = u32::MAX - 1;
    const THIRD: u16 = u16::MAX - 1;
    const BYTES: &[u8] = &[1, 10, 20, 50, 100];

    let client = Socket::req0().expect("Create client");
    let server = Socket::rep0().expect("Create server");

    server.listen(ADDR.into()).expect("listen");
    client.connect(ADDR.into()).expect("connect");

    let mut req = Message::new().expect("Create message");
    req.append(BYTES).expect("append bytes");
    req.append_u64(FIRST).expect("Apped u64");
    req.append_u32(SECOND).expect("Append u32");
    req.append_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    let req = client.send_msg_async(req).expect("Create send message future");
    rt::run(req).expect("Success");

    let resp = server.recv_msg_async().expect("create future");
    resp.cancel();
    let error = rt::run(resp).expect_err("Should error out");
    assert!(error.is_cancelled());

    let resp = server.recv_msg_async().expect("create future");
    let mut resp = rt::run(resp).expect("Get message").expect("To have message");
    let third = resp.pop_u16().expect("get u16");
    let second = resp.pop_u32().expect("get u32");
    let first = resp.pop_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
    resp.clear();

    let mut req = Message::new().expect("Create message");
    req.insert(BYTES).expect("append bytes");
    req.insert_u64(FIRST).expect("Apped u64");
    req.insert_u32(SECOND).expect("Append u32");
    req.insert_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    client.send_msg(req).expect("Send message");

    let mut resp = server.recv_msg().expect("Get message");
    let third = resp.pop_front_u16().expect("get u16");
    let second = resp.pop_front_u32().expect("get u32");
    let first = resp.pop_front_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
}

#[test]
fn should_do_req_resp_async_ipc() {
    const ADDR: &str =  "ipc://should_do_req_resp_async_ipc\0";

    const FIRST: u64 = u64::MAX - 1;
    const SECOND: u32 = u32::MAX - 1;
    const THIRD: u16 = u16::MAX - 1;
    const BYTES: &[u8] = &[1, 10, 20, 50, 100];

    let client = Socket::req0().expect("Create client");
    let server = Socket::rep0().expect("Create server");

    server.listen(ADDR.into()).expect("listen");
    client.connect(ADDR.into()).expect("connect");

    let mut req = Message::new().expect("Create message");
    req.append(BYTES).expect("append bytes");
    req.append_u64(FIRST).expect("Apped u64");
    req.append_u32(SECOND).expect("Append u32");
    req.append_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    let req = client.send_msg_async(req).expect("Create send message future");
    rt::run(req).expect("Success");

    let resp = server.recv_msg_async().expect("create future");
    let mut resp = rt::run(resp).expect("Get message").expect("To have message");
    let third = resp.pop_u16().expect("get u16");
    let second = resp.pop_u32().expect("get u32");
    let first = resp.pop_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
    resp.clear();

    let mut req = Message::new().expect("Create message");
    req.insert(BYTES).expect("append bytes");
    req.insert_u64(FIRST).expect("Apped u64");
    req.insert_u32(SECOND).expect("Append u32");
    req.insert_u16(THIRD).expect("Append u16");

    let resp = server.try_recv_msg().expect("Attempt to peek");
    assert!(resp.is_none());

    client.send_msg(req).expect("Send message");

    let mut resp = server.recv_msg().expect("Get message");
    let third = resp.pop_front_u16().expect("get u16");
    let second = resp.pop_front_u32().expect("get u32");
    let first = resp.pop_front_u64().expect("get u64");

    assert_eq!(first, FIRST);
    assert_eq!(second, SECOND);
    assert_eq!(third, THIRD);
    assert_eq!(resp.body(), BYTES);
}
