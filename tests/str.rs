use nng_c::str::{String, STATIC_SIZE};

#[test]
fn should_create_valid_static_string() {
    let strs = [
        "\0",
        "1\0",
        "12\0",
    ];

    for str in strs {
        assert_eq!(String::new(str.as_bytes()), str.as_bytes()[..str.len()-1])
    }
}

#[test]
fn should_create_valid_dynamic_str() {
    let mut string = Vec::with_capacity(STATIC_SIZE);

    let mut cursor = 0;
    while cursor <= STATIC_SIZE {
        string.push(b'a' + (cursor % 10) as u8);
        cursor += 1;
        let str = String::new(string.as_slice());
        assert_eq!(str, string.as_slice());
    }
}
