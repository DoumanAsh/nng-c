use nng_c::{Socket, Message, NngError};
use nng_c::tls::{self, Config};

mod rt;

const CERT: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIDRzCCAi8CFCOIJGs6plMawgBYdDuCRV7UuJuyMA0GCSqGSIb3DQEBCwUAMF8x
CzAJBgNVBAYTAlhYMQ8wDQYDVQQIDAZVdG9waWExETAPBgNVBAcMCFBhcmFkaXNl
MRgwFgYDVQQKDA9OTkcgVGVzdHMsIEluYy4xEjAQBgNVBAMMCWxvY2FsaG9zdDAg
Fw0yMDA1MjMyMzMxMTlaGA8yMTIwMDQyOTIzMzExOVowXzELMAkGA1UEBhMCWFgx
DzANBgNVBAgMBlV0b3BpYTERMA8GA1UEBwwIUGFyYWRpc2UxGDAWBgNVBAoMD05O
RyBUZXN0cywgSW5jLjESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0B
AQEFAAOCAQ8AMIIBCgKCAQEAyPdnRbMrQj9902TGQsmMbG6xTSl9XKbJr55BcnyZ
ifsrqA7BbNSkndVw9Qq+OJQIDBTfRhGdG+o9j3h6SDVvIb62fWtwJ5Fe0eUmeYwP
c1PKQzOmMFlMYekXiZsx60yu5LeuUhGlb84+csImH+m3NbutInPJcStSq0WfSV6V
Nk6DN3535ex66zV2Ms6ikys1vCC434YqIpe1VxUh+IC2widJcLDCxmmJt3TOlx5f
9OcKMkxuH4fMAzgjIEpIrUjdb19CGNVvsNrEEB2CShBMgBdqMaAnKFxpKgfzS0JF
ulxRGNtpsrweki+j+a4sJXTv40kELkRQS6uB6wWZNjcPywIDAQABMA0GCSqGSIb3
DQEBCwUAA4IBAQA86Fqrd4aiih6R3fwiMLwV6IQJv+u5rQeqA4D0xu6v6siP42SJ
YMaI2DkNGrWdSFVSHUK/efceCrhnMlW7VM8I1cyl2F/qKMfnT72cxqqquiKtQKdT
NDTzv61QMUP9n86HxMzGS7jg0Pknu55BsIRNK6ndDvI3D/K/rzZs4xbqWSSfNfQs
fNFBbOuDrkS6/1h3p8SY1uPM18WLVv3GO2T3aeNMHn7YJAKSn+sfaxzAPyPIK3UT
W8ecGQSHOqBJJQELyUfMu7lx/FCYKUhN7/1uhU5Qf1pCR8hkIMegtqr64yVBNMOn
248fuiHbs9BRknuA/PqjxIDDZTwtDrfVSO/S
-----END CERTIFICATE-----\0";

const KEY: &[u8] = b"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEAyPdnRbMrQj9902TGQsmMbG6xTSl9XKbJr55BcnyZifsrqA7B
bNSkndVw9Qq+OJQIDBTfRhGdG+o9j3h6SDVvIb62fWtwJ5Fe0eUmeYwPc1PKQzOm
MFlMYekXiZsx60yu5LeuUhGlb84+csImH+m3NbutInPJcStSq0WfSV6VNk6DN353
5ex66zV2Ms6ikys1vCC434YqIpe1VxUh+IC2widJcLDCxmmJt3TOlx5f9OcKMkxu
H4fMAzgjIEpIrUjdb19CGNVvsNrEEB2CShBMgBdqMaAnKFxpKgfzS0JFulxRGNtp
srweki+j+a4sJXTv40kELkRQS6uB6wWZNjcPywIDAQABAoIBAQCGSUsot+BgFCzv
5JbWafb7Pbwb421xS8HZJ9Zzue6e1McHNVTqc+zLyqQAGX2iMMhvykKnf32L+anJ
BKgxOANaeSVYCUKYLfs+JfDfp0druMGexhR2mjT/99FSkfF5WXREQLiq/j+dxiLU
bActq+5QaWf3bYddp6VF7O/TBvCNqBfD0+S0o0wtBdvxXItrKPTD5iKr9JfLWdAt
YNAk2QgFywFtY5zc2wt4queghF9GHeBzzZCuVj9QvPA4WdVq0mePaPTmvTYQUD0j
GT6X5j9JhqCwfh7trb/HfkmLHwwc62zPDFps+Dxao80+vss5b/EYZ4zY3S/K3vpG
f/e42S2BAoGBAP51HQYFJGC/wsNtOcX8RtXnRo8eYmyboH6MtBFrZxWl6ERigKCN
5Tjni7EI3nwi3ONg0ENPFkoQ8h0bcVFS7iW5kz5te73WaOFtpkU9rmuFDUz37eLP
d+JLZ5Kwfn2FM9HoiSAZAHowE0MIlmmIEXSnFtqA2zzorPQLO/4QlR+VAoGBAMov
R0yaHg3qPlxmCNyLXKiGaGNzvsvWjYw825uCGmVZfhzDhOiCFMaMb51BS5Uw/gwm
zHxmJjoqak8JjxaQ1qKPoeY1TJ5ps1+TRq9Wzm2/zGqJHOXnRPlqwBQ6AFllAMgt
Rlp5uqb8QJ+YEo6/1kdGhw9kZWCZEEue6MNQjxnfAoGARLkUkZ+p54di7qz9QX+V
EghYgibOpk6R1hviNiIvwSUByhZgbvxjwC6pB7NBg31W8wIevU8K0g4plbrnq/Md
5opsPhwLo4XY5albkq/J/7f7k6ISWYN2+WMsIe4Q+42SJUsMXeLiwh1h1mTnWrEp
JbxK69CJZbXhoDe4iDGqVNECgYAjlgS3n9ywWE1XmAHxR3osk1OmRYYMfJv3VfLV
QSYCNqkyyNsIzXR4qdkvVYHHJZNhcibFsnkB/dsuRCFyOFX+0McPLMxqiXIv3U0w
qVe2C28gRTfX40fJmpdqN/c9xMBJe2aJoClRIM8DCBIkG/HMI8a719DcGrS6iqKv
VeuKAwKBgEgD+KWW1KtoSjCBlS0NP8HjC/Rq7j99YhKE6b9h2slIa7JTO8RZKCa0
qbuomdUeJA3R8h+5CFkEKWqO2/0+dUdLNOjG+CaTFHaUJevzHOzIjpn+VsfCLV13
yupGzHG+tGtdrWgLn9Dzdp67cDfSnsSh+KODPECAAFfo+wPvD8DS
-----END RSA PRIVATE KEY-----\0";

#[test]
fn should_do_req_resp_async_tls() {
    const ADDR: &str =  "tls+tcp://localhost:65001\0";

    const FIRST: u64 = u64::MAX - 1;
    const SECOND: u32 = u32::MAX - 1;
    const THIRD: u16 = u16::MAX - 1;
    const BYTES: &[u8] = &[1, 10, 20, 50, 100];

    nng_c::utils::enable_logging(nng_c::utils::Level::Trace);

    let own_cert = tls::OwnCert {
        cert: CERT.into(),
        key: KEY.into(),
        pass: None,
    };

    let ca_cert = tls::CA {
        cert: CERT.into(),
        crl: None,
    };

    let client_config = Config::client().expect("crate config");
    client_config.versions(tls::Version::Tls1_2, tls::Version::Tls1_3).expect("set versions");
    client_config.server_name("localhost").expect("to set server name");
    client_config.auth_mode(tls::Auth::Required).expect("to set auth mode");
    client_config.own_cert(&own_cert).expect("to set own cert");
    client_config.ca_cert(&ca_cert).expect("to set own cert");

    let server_config = Config::server().expect("crate config");
    server_config.versions(tls::Version::Tls1_2, tls::Version::Tls1_3).expect("set versions");
    server_config.own_cert(&own_cert).expect("to set own cert");
    server_config.auth_mode(tls::Auth::Required).expect("to set auth mode");
    server_config.ca_cert(&ca_cert).expect("to set own cert");

    let client = Socket::req0().expect("Create client");
    let server = Socket::rep0().expect("Create server");

    server.listen_with(ADDR.into(), &server_config).expect("listen");
    let options = nng_c::socket::ConnectOptions::new().with_dialer(client_config);
    client.connect_with(ADDR.into(), options).expect("connect");

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
