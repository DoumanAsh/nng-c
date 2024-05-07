use crate::ErrorCode;
use crate::error::error;
use crate::msg::Message;

use crate::sys;
use crate::url::Url;
use sys::nng_socket;
use sys::nng_close;
use sys::{nng_pub0_open, nng_sub0_open};
use sys::{nng_req0_open, nng_rep0_open};

use core::{mem, fmt, ops, ptr};

type InitFn = unsafe extern "C" fn(msg: *mut nng_socket) -> core::ffi::c_int;

#[repr(transparent)]
///Generic socket type
pub struct Socket(pub(crate) nng_socket);

impl Socket {
    #[inline]
    fn with(init: InitFn) -> Result<Self, ErrorCode> {
        let mut socket = nng_socket {
            id: 0
        };

        let result = unsafe {
            (init)(&mut socket)
        };

        if result == 0 {
            Ok(Self(socket))
        } else {
            Err(error(result))
        }
    }

    #[inline(always)]
    ///Creates new version 0 publisher socket
    pub fn pub0() -> Result<Self, ErrorCode> {
        Self::with(nng_pub0_open)
    }

    #[inline(always)]
    ///Creates new version 0 subscriber socket
    pub fn sub0() -> Result<Self, ErrorCode> {
        Self::with(nng_sub0_open)
    }

    #[inline(always)]
    ///Creates new version 0 request socket
    pub fn req0() -> Result<Self, ErrorCode> {
        Self::with(nng_req0_open)
    }

    #[inline(always)]
    ///Creates new version 0 reply socket
    pub fn rep0() -> Result<Self, ErrorCode> {
        Self::with(nng_rep0_open)
    }

    #[inline(always)]
    ///Closes socket.
    ///
    ///Returns `true` if operation had effect
    ///Otherwise, if socket is already closed, returns `false`
    pub fn close(&self) -> bool {
        unsafe {
            nng_close(self.0) == 0
        }
    }

    #[inline]
    ///Binds socket to the specified `url`, starting to listen for incoming messages.
    pub fn listen(&self, url: Url<'_>) -> Result<(), ErrorCode> {
        let url = url.as_ptr();
        let result = unsafe {
            sys::nng_listen(**self, url as _, ptr::null_mut(), 0)
        };
        match result {
            0 => Ok(()),
            code => Err(error(code))
        }
    }

    #[inline]
    ///Connects to the remote peer via `url`.
    pub fn connect(&self, url: Url<'_>) -> Result<(), ErrorCode> {
        let url = url.as_ptr();
        let result = unsafe {
            sys::nng_dial(**self, url as _, ptr::null_mut(), 0)
        };
        match result {
            0 => Ok(()),
            code => Err(error(code))
        }
    }

    #[inline]
    ///Tries to get message, if available, returning `None` if no message is available
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    pub fn try_recv_msg(&self) -> Result<Option<Message>, ErrorCode> {
        let mut msg = ptr::null_mut();
        let result = unsafe {
            sys::nng_recvmsg(**self, &mut msg, sys::NNG_FLAG_NONBLOCK)
        };

        match ptr::NonNull::new(msg) {
            Some(ptr) => Ok(Some(Message(ptr))),
            None => match result {
                sys::nng_errno_enum::NNG_EAGAIN => Ok(None),
                code => Err(error(code)),
            }
        }
    }

    #[inline]
    ///Receives pending message, waiting forever if none is available.
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    pub fn recv_msg(&self) -> Result<Message, ErrorCode> {
        let mut msg = ptr::null_mut();
        let result = unsafe {
            sys::nng_recvmsg(**self, &mut msg, 0)
        };

        match ptr::NonNull::new(msg) {
            Some(ptr) => Ok(Message(ptr)),
            None => Err(error(result)),
        }
    }

    #[inline]
    ///Sends message over the socket.
    ///
    ///If successful takes ownership of message.
    ///Otherwise returns message with error code.
    pub fn send_msg(&self, msg: Message) -> Result<(), (Message, ErrorCode)> {
        let result = unsafe {
            sys::nng_sendmsg(**self, msg.as_ptr(), 0)
        };

        match result {
            0 => {
                mem::forget(msg);
                Ok(())
            },
            code => Err((msg, error(code))),
        }
    }
}

impl fmt::Debug for Socket {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_fmt(format_args!("Socket(id={})", self.0.id))
    }
}

impl Drop for Socket {
    #[inline(always)]
    fn drop(&mut self) {
        self.close();
    }
}

impl ops::Deref for Socket {
    type Target = nng_socket;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Socket {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
