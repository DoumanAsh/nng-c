//! Options

use crate::sys;
use crate::socket::Socket;
use crate::error::{error, ErrorCode};

use core::time;
use core::convert::TryInto;

///Options interface
pub trait Options<T> {
    ///Applies options to the target, returning error if any happens
    fn apply(&self, target: &T) -> Result<(), ErrorCode>;
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

macro_rules! set_duration_option {
    ($socket:expr, $name:expr, $duration:expr) => {
        match $duration.as_millis().try_into() {
            Ok(duration) => unsafe {
                match sys::nng_socket_set_ms($socket, $name.as_ptr() as _, duration) {
                    0 => (),
                    code => return Err(error(code)),
                }
            },
            Err(_) => return Err(error(sys::nng_errno_enum::NNG_EINVAL)),
        }
    }
}

#[derive(Copy, Clone, Default)]
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
            set_duration_option!(**target, sys::NNG_OPT_REQ_RESENDTIME, resend_time);
        }

        if let Some(resend_tick) = self.resend_tick {
            set_duration_option!(**target, sys::NNG_OPT_REQ_RESENDTICK, resend_tick);
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Default)]
///Topic to subscribe to for sub protocol.
pub struct Subscribe<'a>(pub &'a [u8]);

impl Options<Socket> for Subscribe<'_> {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_bytes_option!(**target, sys::NNG_OPT_SUB_SUBSCRIBE, self.0)
    }
}

#[derive(Copy, Clone, Default)]
///Topic to unsubscribe from for sub protocol.
pub struct Unsubscribe<'a>(pub &'a [u8]);

impl Options<Socket> for Unsubscribe<'_> {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_bytes_option!(**target, sys::NNG_OPT_SUB_UNSUBSCRIBE, self.0)
    }
}

#[derive(Copy, Clone, Default)]
///Max number of hops message can make to reach peer
///
///Usually defaults to 8
pub struct MaxTtl(pub u8);

impl Options<Socket> for MaxTtl {
    fn apply(&self, target: &Socket) -> Result<(), ErrorCode> {
        set_int_option!(**target, sys::NNG_OPT_MAXTTL, self.0)
    }
}
