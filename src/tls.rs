//!TLS module

use crate::socket;
use crate::str::String;
use crate::options::Options;
use crate::defs::MAX_HOSTNAME_LEN;
use crate::error::{error, ErrorCode};

use core::ptr::{self, NonNull};
use core::ffi::CStr;

use nng_c_sys as sys;
use sys::{nng_tls_mode, nng_tls_auth_mode, nng_tls_version};
use sys::nng_tls_config;

///Get available TLS engine
pub fn get_engine_name() -> &'static str {
    //This never fails
    let name = unsafe {
        CStr::from_ptr(
            nng_c_sys::nng_tls_engine_name()
        )
    };

    name.to_str().unwrap_or("unknown")
}

#[derive(Debug, Clone)]
///Certificate authority to validate remote peer
pub struct CA<'a> {
    ///PEM encoded certificate or chain
    pub cert: String<'a>,
    ///Optional PEM encoded revocation list
    pub crl: Option<String<'a>>,
}

#[derive(Debug, Clone)]
///Local certificate input
pub struct OwnCert<'a> {
    ///PEM encoded certificate or chain
    pub cert: String<'a>,
    ///PEM encoded private key.
    pub key: String<'a>,
    ///Optional passphrase to decrypt private key.
    pub pass: Option<String<'a>>,
}

///Authentication mode
#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum Auth {
    ///No authentication of the TLS peer is performed. This is the default for TLS servers, which most typically do not authenticate their clients.
    None = nng_tls_auth_mode::NNG_TLS_AUTH_MODE_NONE,
    ///If a certificate is presented by the peer, then it is validated. However, if the peer does not present a valid certificate, then the session is allowed to proceed without authentication.
    Optional = nng_tls_auth_mode::NNG_TLS_AUTH_MODE_OPTIONAL,
    ///A check is made to ensure that the peer has presented a valid certificate used for the session. If the peer’s certificate is invalid or missing, then the session is refused. This is the default for clients.
    Required = nng_tls_auth_mode::NNG_TLS_AUTH_MODE_REQUIRED,
}

///TLS version
#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum Version {
    ///TLS 1.0
    Tls1_0 = nng_tls_version::NNG_TLS_1_0,
    ///TLS 1.1
    Tls1_1 = nng_tls_version::NNG_TLS_1_1,
    ///TLS 1.2
    Tls1_2 = nng_tls_version::NNG_TLS_1_2,
    ///TLS 1.3
    Tls1_3 = nng_tls_version::NNG_TLS_1_3,
}

///TLS Config container
///
///This object is reference counted pointer so it can be cloned safely.
///
///It can be applied when listening or connecting.
pub struct Config(NonNull<nng_tls_config>);

impl Config {
    fn new(mode: nng_tls_mode::Type) -> Option<Self> {
        let mut ptr = ptr::null_mut();
        unsafe {
            sys::nng_tls_config_alloc(&mut ptr, mode);
        }

        NonNull::new(ptr).map(Self)
    }

    #[inline(always)]
    ///Creates client configuration.
    ///
    ///Returns `None` when cannot allocate memory
    pub fn client() -> Option<Self> {
        Self::new(nng_tls_mode::NNG_TLS_MODE_CLIENT)
    }

    #[inline(always)]
    ///Creates server configuration.
    ///
    ///Returns `None` when cannot allocate memory
    pub fn server() -> Option<Self> {
        Self::new(nng_tls_mode::NNG_TLS_MODE_SERVER)
    }

    ///Sets `Auth` mode of the configuration
    ///
    ///Defaults:
    ///- None for server side (listener)
    ///- Required for client side (connecting)
    pub fn auth_mode(&self, mode: Auth) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_tls_config_auth_mode(self.0.as_ptr(), mode as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    ///Sets range of supported TLS `Version`s
    pub fn versions(&self, min: Version, max: Version) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_tls_config_version(self.0.as_ptr(), min as _, max as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    ///Sets server `name` for the client connection
    pub fn server_name(&self, name: &str) -> Result<(), ErrorCode> {
        if name.len() > MAX_HOSTNAME_LEN {
            return Err(error(sys::nng_errno_enum::NNG_EADDRINVAL));
        }

        let mut buffer = [0u8; MAX_HOSTNAME_LEN + 1];
        let result = unsafe {
            ptr::copy_nonoverlapping(name.as_ptr(), buffer.as_mut_ptr(), name.len());
            sys::nng_tls_config_server_name(self.0.as_ptr(), buffer.as_ptr() as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    ///Sets CA certificate used in TLS handshake
    pub fn ca_cert(&self, cert: &CA<'_>) -> Result<(), ErrorCode> {
        let crl = match cert.crl.as_ref() {
            Some(crl) => crl.as_ptr(),
            None => ptr::null()
        };
        let cert = cert.cert.as_ptr();
        let result = unsafe {
            sys::nng_tls_config_ca_chain(self.0.as_ptr(), cert as _, crl as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    ///Sets local certificate used in TLS handshake
    pub fn own_cert(&self, cert: &OwnCert<'_>) -> Result<(), ErrorCode> {
        let pass = match cert.pass.as_ref() {
            Some(pass) => pass.as_ptr(),
            None => ptr::null()
        };
        let key = cert.key.as_ptr();
        let cert = cert.cert.as_ptr();
        let result = unsafe {
            sys::nng_tls_config_own_cert(self.0.as_ptr(), cert as _, key as _, pass as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }
}

impl Clone for Config {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            sys::nng_tls_config_hold(self.0.as_ptr())
        }
        Self(self.0)
    }
}

impl Drop for Config {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            sys::nng_tls_config_free(self.0.as_ptr())
        }
    }
}

impl Options<socket::Listener> for Config {
    #[inline]
    fn apply(&self, target: &socket::Listener) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_listener_set_ptr(target.0, sys::NNG_OPT_TLS_CONFIG.as_ptr() as _, self.0.as_ptr() as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }
}

impl Options<socket::Dialer> for Config {
    #[inline]
    fn apply(&self, target: &socket::Dialer) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_dialer_set_ptr(target.0, sys::NNG_OPT_TLS_CONFIG.as_ptr() as _, self.0.as_ptr() as _)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }
}
