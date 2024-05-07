use nng_c::url::{Url, STATIC_SIZE};

#[test]
fn should_create_valid_static_url() {
    let urls = [
        "\0",
        "1\0",
        "12\0",
    ];

    for url in urls {
        assert_eq!(Url::new(url.as_bytes()), url.as_bytes()[..url.len()-1])
    }
}

#[test]
fn should_create_valid_dynamic_url() {
    let mut string = Vec::with_capacity(STATIC_SIZE);

    let mut cursor = 0;
    while cursor <= STATIC_SIZE {
        string.push(b'a' + (cursor % 10) as u8);
        cursor += 1;
        let url = Url::new(string.as_slice());
        assert_eq!(url, string.as_slice());
    }
}
