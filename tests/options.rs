use nng_c::{options, Socket};

#[test]
fn should_set_and_read_socket_name() {
    const NAME: &str = "my_socket_name";
    let option = options::SocketName::new(NAME).expect("to fit name");

    let socket = Socket::req0().expect("Create client");

    let name: options::SocketName = socket.get_prop().expect("To get socket name");
    assert_ne!(name, option);

    socket.set_opt(option).expect("set socket name");

    let name: options::SocketName = socket.get_prop().expect("To get socket name");
    assert_eq!(name, option);
    assert_eq!(name, NAME);
}

#[test]
fn should_read_peer_name() {
    const ADDR: &str =  "inproc://should_set_and_read_socket_name\0";

    let client = Socket::req0().expect("Create client");
    let server = Socket::rep0().expect("Create server");

    server.listen(ADDR.into()).expect("listen");
    client.connect(ADDR.into()).expect("connect");

    let mut peer: options::PeerName = server.get_prop().expect("get peer name");
    assert_eq!("req", peer);
    peer = client.get_prop().expect("get peer name");
    assert_eq!("rep", peer);
}
