//!NNG error definition

use core::ptr;
use core::ffi::c_int;
use core::ffi::CStr;

pub use error_code::ErrorCode;

use crate::sys;

///Extension to error code with shortcut for some meaningful checks
pub trait NngError {
    ///Returns whether error code indicates cancellation of future.
    fn is_cancelled(&self) -> bool;
    ///Returns whether error code indicates operation timed out.
    fn is_timed_out(&self) -> bool;
    ///Returns whether error code indicates aborted connection.
    fn is_conn_aborted(&self) -> bool;
    ///Returns whether error code indicates connection has been reset.
    fn is_conn_reset(&self) -> bool;
    ///Returns whether error code indicates connection has been refused.
    fn is_conn_refused(&self) -> bool;
    ///Returns whether error code indicates problem with peer's authentication
    fn is_peer_auth(&self) -> bool;
    ///Returns whether error code indicates problem using crypto
    ///
    ///This is mostly indicates invalid local configuration (i.e. no TLS certificate etc)
    fn is_crypto(&self) -> bool;
}

impl NngError for ErrorCode {
    #[inline(always)]
    fn is_cancelled(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ECANCELED
    }

    #[inline(always)]
    fn is_timed_out(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ETIMEDOUT
    }

    #[inline(always)]
    fn is_conn_aborted(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ECONNABORTED
    }

    #[inline(always)]
    fn is_conn_reset(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ECONNRESET
    }

    #[inline(always)]
    fn is_conn_refused(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ECONNREFUSED
    }

    #[inline(always)]
    fn is_peer_auth(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_EPEERAUTH
    }

    #[inline(always)]
    fn is_crypto(&self) -> bool {
        self.raw_code() == sys::nng_errno_enum::NNG_ECRYPTO
    }
}

static CATEGORY: error_code::Category = error_code::Category {
    name: "NngError",
    equivalent,
    is_would_block,
    message,
};

fn equivalent(code: c_int, other: &ErrorCode) -> bool {
    ptr::eq(&CATEGORY, other.category()) && code == other.raw_code()
}

fn is_would_block(code: c_int) -> bool {
    code == nng_c_sys::nng_errno_enum::NNG_EAGAIN
}

fn message(code: c_int, _: &mut error_code::MessageBuf) -> &str {
    //nng returns static buffer or constant therefore there is no need to copy message
    let msg = unsafe {
        CStr::from_ptr(
            nng_c_sys::nng_strerror(code)
        )
    };

    match msg.to_str() {
        Ok(msg) => msg,
        Err(_) => "Non-utf8 error message",
    }
}

#[cold]
#[inline(never)]
///Creates new nng error
pub(crate) fn error(code: c_int) -> ErrorCode {
    ErrorCode::new(code, &CATEGORY)
}
