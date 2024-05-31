//! Options

use crate::sys;
use crate::socket::Socket;
use crate::error::{error, ErrorCode};

use core::{fmt, time};
use core::convert::TryInto;

///Property interface
pub trait Property<T>: Sized {
    ///Gets instance of self from the `target
    fn get(target: &T) -> Result<Self, ErrorCode>;
}

///Options interface
pub trait Options<T> {
    ///Applies options to the target, returning error if any happens
    fn apply(&self, target: &T) -> Result<(), ErrorCode>;
}

impl<T> Options<T> for () {
    #[inline(always)]
    fn apply(&self, _: &T) -> Result<(), ErrorCode> {
        Ok(())
    }
}

macro_rules! set_bytes_option {
    ($socket:expr, $name:expr, $bytes:expr) => {
        unsafe {
            let bytes = $bytes;
            match sys::nng_socket_set($socket, $name.as_ptr() as _, bytes.as_ptr() as _, bytes.len()) {
                0 => Ok(()),
                code => Err(error(code)),
            }
        }
    }
}

macro_rules! set_string_option {
    ($socket:expr, $name:expr, $bytes:expr) => {
        unsafe {
            let bytes = $bytes;
            match sys::nng_socket_set_string($socket, $name.as_ptr() as _, bytes.as_ptr() as _) {
                0 => Ok(()),
                code => Err(error(code)),
            }
        }
    }
}

macro_rules! set_int_option {
    ($socket:expr, $name:expr, $num:expr) => {
        unsafe {
            match sys::nng_socket_set_int($socket, $name.as_ptr() as _, $num as _) {
                0 => Ok(()),
                code => Err(error(code)),
            }
        }
    }
}

macro_rules! set_size_t_option {
    ($socket:expr, $name:expr, $num:expr) => {
        unsafe {
            match sys::nng_socket_set_size($socket, $name.as_ptr() as _, $num as _) {
                0 => Ok(()),
                code => Err(error(code)),
            }
        }
    }
}

macro_rules! set_duration_option {
    ($socket:expr, $name:expr, $duration:expr) => {
        match $duration.as_millis().try_into() {
            Ok(duration) => unsafe {
                match sys::nng_socket_set_ms($socket, $name.as_ptr() as _, duration) {
                    0 => Ok(()),
                    code => Err(error(code)),
                }
            },
            Err(_) => Err(error(sys::nng_errno_enum::NNG_EINVAL)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
///Req protocol options
pub struct Req {
    ///Duration after which request is considered failed to be delivered
    ///Therefore triggering re-sending
    pub resend_time: Option<time::Duration>,
    ///Granularity of the clock used to check for resending time
    pub resend_tick: Option<time::Duration>,
}

impl Options<Socket> for Req {
    #[inline]
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        if let Some(resend_time) = self.resend_time {
            set_duration_option!(**target, sys::NNG_OPT_REQ_RESENDTIME, resend_time)?;
        }

        if let Some(resend_tick) = self.resend_tick {
            set_duration_option!(**target, sys::NNG_OPT_REQ_RESENDTICK, resend_tick)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
///Topic to subscribe to for sub protocol.
pub struct Subscribe<'a>(pub &'a [u8]);

impl Options<Socket> for Subscribe<'_> {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_bytes_option!(**target, sys::NNG_OPT_SUB_SUBSCRIBE, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Topic to unsubscribe from for sub protocol.
pub struct Unsubscribe<'a>(pub &'a [u8]);

impl Options<Socket> for Unsubscribe<'_> {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_bytes_option!(**target, sys::NNG_OPT_SUB_UNSUBSCRIBE, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Max number of hops message can make to reach peer
///
///Usually defaults to 8
pub struct MaxTtl(pub u8);

impl Options<Socket> for MaxTtl {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_int_option!(**target, sys::NNG_OPT_MAXTTL, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Reconnect options
pub struct Reconnect {
    ///This is the minimum amount of time to wait before attempting to establish a connection after a previous attempt has failed
    pub min_time: Option<time::Duration>,
    ///This is the maximum amount of time to wait before attempting to establish a connection after a previous attempt has failed
    ///
    ///This can be set to 0, to disable exponential back-off
    pub max_time: Option<time::Duration>,
}

impl Options<Socket> for Reconnect {
    #[inline]
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        if let Some(min_time) = self.min_time {
            set_duration_option!(**target, sys::NNG_OPT_RECONNMINT, min_time)?;
        }

        if let Some(max_time) = self.max_time {
            set_duration_option!(**target, sys::NNG_OPT_RECONNMAXT, max_time)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
///Sets internal receive buffer to this amount of messages
///
///Allowed values are from 0 to 8192.
pub struct RecvBuf(pub u16);

impl Options<Socket> for RecvBuf {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_int_option!(**target, sys::NNG_OPT_RECVBUF, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Limits size of message that socket can receive
///
///This specifically limits byte size of message, rejecting any attempt sending receiving of size beyond the limit.
pub struct RecvMaxSize(pub usize);

impl Options<Socket> for RecvMaxSize {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_size_t_option!(**target, sys::NNG_OPT_RECVMAXSZ, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Sets timeout on message receive.
///
///If no message is available within specified time, then it shall error out with timed_out error
pub struct RecvTimeout(pub time::Duration);

impl Options<Socket> for RecvTimeout {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_duration_option!(**target, sys::NNG_OPT_RECVTIMEO, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Sets internal send buffer to this amount of messages
///
///Allowed values are from 0 to 8192.
pub struct SendBuf(pub u16);

impl Options<Socket> for SendBuf {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_int_option!(**target, sys::NNG_OPT_SENDBUF, self.0)
    }
}

#[derive(Copy, Clone, Debug)]
///Sets timeout on message send.
///
///If message cannot be sent within specified time, then it shall error out with timed_out error
pub struct SendTimeout(pub time::Duration);

impl Options<Socket> for SendTimeout {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_duration_option!(**target, sys::NNG_OPT_SENDTIMEO, self.0)
    }
}

#[derive(Copy, Clone, Eq)]
///Socket name, limited to 63 characters.
///
///This is purely informative property without any functional use by nng itself
pub struct SocketName(pub(crate) [u8; 64]);

impl SocketName {
    ///Creates new name, returning `Some` if input fits 63 characters limit
    pub fn new(name: &str) -> Option<Self> {
        let mut buf = [0; 64];
        if name.len() < buf.len() {
            buf[..name.len()].copy_from_slice(name.as_bytes());
            Some(Self(buf))
        } else {
            None
        }
    }

    ///Access raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        if let Some(idx) = self.0.iter().position(|byt| *byt == 0) {
            &self.0[..idx]
        } else {
            &self.0
        }
    }

    ///Returns string, if raw bytes are valid unicode
    pub fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.as_bytes()).ok()
    }
}

impl Options<Socket> for SocketName {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_string_option!(**target, sys::NNG_OPT_SOCKNAME, self.0)
    }
}

impl Property<Socket> for SocketName {
    fn get(target: &Socket) -> Result<Self, ErrorCode> {
        let mut buf = [0; 64];
        let result = unsafe {
            sys::nng_socket_get(**target, sys::NNG_OPT_SOCKNAME.as_ptr() as _, buf.as_mut_ptr() as _, &mut buf.len())
        };

        match result {
            0 => Ok(Self(buf)),
            code => Err(error(code))
        }
    }
}

impl PartialEq for SocketName {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<PeerName> for SocketName {
    #[inline]
    fn eq(&self, other: &PeerName) -> bool {
        self.as_bytes() == other.0.as_bytes()
    }
}

impl PartialEq<str> for SocketName {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for SocketName {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<SocketName> for str {
    #[inline]
    fn eq(&self, other: &SocketName) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<SocketName> for &str {
    #[inline]
    fn eq(&self, other: &SocketName) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl fmt::Debug for SocketName {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_tuple("SocketName");
        match self.as_str() {
            Some(name) => fmt.field(&name).finish(),
            None => fmt.field(&self.0).finish(),
        }
    }
}

impl fmt::Display for SocketName {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(name) => fmt.write_str(name),
            None => fmt.write_str("<non-utf-8>"),
        }
    }
}

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
///Peer name, limited to 63 characters.
///
///This tells protocol of the peer
pub struct PeerName(pub(crate) SocketName);

impl Property<Socket> for PeerName {
    fn get(target: &Socket) -> Result<Self, ErrorCode> {
        let mut buf = [0; 64];
        let result = unsafe {
            sys::nng_socket_get(**target, sys::NNG_OPT_PEERNAME.as_ptr() as _, buf.as_mut_ptr() as _, &mut buf.len())
        };

        match result {
            0 => Ok(Self(SocketName(buf))),
            code => Err(error(code))
        }
    }
}

impl PartialEq<SocketName> for PeerName {
    #[inline]
    fn eq(&self, other: &SocketName) -> bool {
        self.0.as_bytes() == other.as_bytes()
    }
}

impl PartialEq for PeerName {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes() == other.0.as_bytes()
    }
}

impl PartialEq<str> for PeerName {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for PeerName {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<PeerName> for str {
    #[inline]
    fn eq(&self, other: &PeerName) -> bool {
        self.as_bytes() == other.0.as_bytes()
    }
}

impl PartialEq<PeerName> for &str {
    #[inline]
    fn eq(&self, other: &PeerName) -> bool {
        self.as_bytes() == other.0.as_bytes()
    }
}

impl fmt::Debug for PeerName {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_tuple("PeerName");
        match self.0.as_str() {
            Some(name) => fmt.field(&name).finish(),
            None => fmt.field(&self.0).finish(),
        }
    }
}

impl fmt::Display for PeerName {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_str() {
            Some(name) => fmt.write_str(name),
            None => fmt.write_str("<non-utf-8>"),
        }
    }
}
