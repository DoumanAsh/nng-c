//! Utilities

use nng_c_sys::nng_log_set_level;
use nng_c_sys::nng_system_logger;
use nng_c_sys::nng_null_logger;
use nng_c_sys::nng_log_set_logger;

#[derive(Copy, Clone)]
#[repr(i32)]
///NNG logging level
pub enum Level {
    ///NNG Error
    Error = nng_c_sys::nng_log_level::NNG_LOG_ERR,
    ///NNG Warnings
    Warn = nng_c_sys::nng_log_level::NNG_LOG_WARN,
    ///NNG Notice level
    Info = nng_c_sys::nng_log_level::NNG_LOG_NOTICE,
    ///NNG Info level
    Debug = nng_c_sys::nng_log_level::NNG_LOG_INFO,
    ///NNG Debug level
    Trace = nng_c_sys::nng_log_level::NNG_LOG_DEBUG,
}

impl Level {
    ///Initializes default logging level
    ///
    ///In debug builds it is `Debug` while in release builds it is `Warn`
    pub const fn new() -> Self {
        if cfg!(debug_assertions) {
            Self::Debug
        } else {
            Self::Warn
        }
    }
}

impl Default for Level {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

///Disables logging
pub fn disable_logging() {
    unsafe {
        nng_log_set_logger(Some(nng_null_logger));
    }
}

///Enables logging using standard facilities of nng.
///
///Specifically it will use syslog on POSIX compliant systems while other systems will use stderr
pub fn enable_logging(level: Level) {
    unsafe {
        nng_log_set_level(level as _);
        nng_log_set_logger(Some(nng_system_logger));
    }
}

#[cfg(feature = "log")]
#[cfg_attr(docsrs, doc(cfg(feature = "log")))]
///Enables logging using [log](https://crates.io/crates/log) crate
///
///Requires feature `log`
///
///Note that messages are only logged if C strings are valid utf-8
pub fn enable_log_logging(level: Level) {
    use core::ffi::CStr;

    unsafe extern "C" fn nng_rust_log_logger(level: nng_c_sys::nng_log_level::Type, _: nng_c_sys::nng_log_facility::Type, msg_id: *const core::ffi::c_char, msg: *const core::ffi::c_char) {
        const NNG: &str = "NNG";

        if msg.is_null() {
            return;
        }

        let level = match level {
            nng_c_sys::nng_log_level::NNG_LOG_DEBUG => log::Level::Trace,
            nng_c_sys::nng_log_level::NNG_LOG_INFO => log::Level::Debug,
            nng_c_sys::nng_log_level::NNG_LOG_NOTICE => log::Level::Info,
            nng_c_sys::nng_log_level::NNG_LOG_WARN => log::Level::Warn,
            nng_c_sys::nng_log_level::NNG_LOG_ERR => log::Level::Error,
            _ => return,
        };

        let msg = match CStr::from_ptr(msg).to_str() {
            Ok(msg) => msg,
            Err(_) => return,
        };

        let nng_tag = if msg_id.is_null() {
            NNG
        } else {
            match CStr::from_ptr(msg_id).to_str() {
                Ok(msg) => msg,
                Err(_) => NNG,
            }
        };

        log::log!(level, "{}: {}", nng_tag, msg);
    }

    unsafe {
        nng_log_set_level(level as _);
        nng_log_set_logger(Some(nng_rust_log_logger));
    }
}
