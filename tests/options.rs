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
