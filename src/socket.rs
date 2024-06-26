//!Socket module
use crate::ErrorCode;
use crate::error::error;
use crate::msg::Message;
use crate::aio::Aio;
use crate::sys;
use crate::str::String;
use crate::options::{Options, Property};

use core::pin::Pin;
use core::ffi::c_int;
use core::future::Future;
use core::{mem, fmt, ops, ptr, task, marker, slice};

use alloc::vec::Vec;

type InitFn = unsafe extern "C" fn(msg: *mut sys::nng_socket) -> core::ffi::c_int;

///Wrapper over slice of bytes.
///
///Can be converted into from any byte slice
pub struct Buf<'a> {
    ptr: *const u8,
    size: usize,
    _lifetime: marker::PhantomData<&'a u8>,
}

impl<'a> Buf<'a> {
    #[inline]
    const fn new(ptr: *const u8, size: usize) -> Self {
        Self {
            ptr,
            size,
            _lifetime: marker::PhantomData,
        }
    }
}

impl<'a> From<&'a [u8]> for Buf<'a> {
    #[inline(always)]
    fn from(value: &'a [u8]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Buf<'a> {
    #[inline(always)]
    fn from(value: &'a [u8; N]) -> Self {
        Self::new(value.as_ptr(), value.len())
    }
}

impl<'a> From<&'a [mem::MaybeUninit<u8>]> for Buf<'a> {
    #[inline(always)]
    fn from(value: &'a [mem::MaybeUninit<u8>]) -> Self {
        Self::new(value.as_ptr() as _, value.len())
    }
}

///Wrapper over mutable slice of bytes.
///
///Can be converted into from any mutable byte slice or mutable Vec
pub struct BufMut<'a> {
    ptr: *mut u8,
    size: usize,
    _lifetime: marker::PhantomData<&'a u8>,
}

impl<'a> BufMut<'a> {
    #[inline]
    const fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            ptr,
            size,
            _lifetime: marker::PhantomData,
        }
    }
}

impl<'a> From<&'a mut Vec<u8>> for BufMut<'a> {
    #[inline(always)]
    fn from(value: &'a mut Vec<u8>) -> Self {
        let value = value.spare_capacity_mut();
        From::from(value)
    }
}

impl<'a> From<&'a mut [u8]> for BufMut<'a> {
    #[inline(always)]
    fn from(value: &'a mut [u8]) -> Self {
        Self::new(value.as_mut_ptr(), value.len())
    }
}

impl<'a, const N: usize> From<&'a mut [u8; N]> for BufMut<'a> {
    #[inline(always)]
    fn from(value: &'a mut [u8; N]) -> Self {
        Self::new(value.as_mut_ptr(), value.len())
    }
}

impl<'a> From<&'a mut [mem::MaybeUninit<u8>]> for BufMut<'a> {
    #[inline(always)]
    fn from(value: &'a mut [mem::MaybeUninit<u8>]) -> Self {
        Self::new(value.as_mut_ptr() as _, value.len())
    }
}

#[derive(Clone, Default)]
///Connect options
pub struct ConnectOptions<T> {
    flags: c_int,
    dialer: T
}

impl ConnectOptions<()> {
    ///Initializes default connect options.
    pub const fn new() -> Self {
        Self {
            flags: 0,
            dialer: ()
        }
    }
}

impl<T> ConnectOptions<T> {
    ///Sets async mode, making connection to be performed in background
    ///
    ///By default, connection blocks, until socket is connected to the remote peer.
    pub const fn with_async(mut self) -> Self {
        self.flags = self.flags | sys::NNG_FLAG_NONBLOCK;
        self
    }

    ///Creates new options with custom dialer options
    ///
    ///This is useful to provide TLS config
    pub const fn with_dialer<R: Options<Dialer>>(&self, dialer: R) -> ConnectOptions<R> {
        ConnectOptions {
            flags: self.flags,
            dialer
        }
    }
}

#[repr(transparent)]
///Generic socket type
pub struct Socket(pub(crate) sys::nng_socket);

impl Socket {
    #[inline(always)]
    fn with(init: InitFn) -> Result<Self, ErrorCode> {
        let mut socket = sys::nng_socket {
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
    ///Creates new version 0 pair socket
    pub fn pair0() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_pair0_open)
    }

    #[inline(always)]
    ///Creates new version 1 pair socket
    pub fn pair1() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_pair1_open)
    }

    #[inline(always)]
    ///Creates new version 0 publisher socket
    pub fn pub0() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_pub0_open)
    }

    #[inline(always)]
    ///Creates new version 0 subscriber socket
    pub fn sub0() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_sub0_open)
    }

    #[inline(always)]
    ///Creates new version 0 request socket
    pub fn req0() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_req0_open)
    }

    #[inline(always)]
    ///Creates new version 0 reply socket
    pub fn rep0() -> Result<Self, ErrorCode> {
        Self::with(sys::nng_rep0_open)
    }

    #[inline(always)]
    ///Closes socket.
    ///
    ///Returns `true` if operation had effect
    ///Otherwise, if socket is already closed, returns `false`
    pub fn close(&self) -> bool {
        unsafe {
            sys::nng_close(self.0) == 0
        }
    }

    #[inline]
    ///Binds socket to the specified `url`, starting to listen for incoming messages.
    pub fn listen(&self, url: String<'_>) -> Result<(), ErrorCode> {
        self.listen_with(url, &())
    }

    #[inline]
    ///Binds socket to the specified `url`, starting to listen for incoming messages.
    ///
    ///Allows to provide custom options to initialize listener with.
    ///Mostly useful to set optional TLS config
    pub fn listen_with<T: Options<Listener>>(&self, url: String<'_>, options: &T) -> Result<(), ErrorCode> {
        let listener = Listener::new(self, url)?;
        options.apply(&listener)?;
        listener.start()?;

        //Listener will be assigned to the socket and can be closed by it
        mem::forget(listener);

        Ok(())
    }

    #[inline]
    ///Connects to the remote peer via `url`.
    pub fn connect(&self, url: String<'_>) -> Result<(), ErrorCode> {
        self.connect_with(url, ConnectOptions::new())
    }

    #[inline]
    ///Connects to the remote peer via `url`, with custom options settings
    pub fn connect_with<T: Options<Dialer>>(&self, url: String<'_>, options: ConnectOptions<T>) -> Result<(), ErrorCode> {
        let dialer = Dialer::new(self, url)?;
        options.dialer.apply(&dialer)?;
        dialer.start(options.flags)?;

        //Dialer will be assigned to the socket and can be closed by it
        mem::forget(dialer);

        Ok(())
    }

    #[inline(always)]
    ///Sets options on the socket
    ///
    ///It is user responsibility to use options that are valid for the protocol of use
    pub fn set_opt<T: Options<Self>>(&self, opts: T) -> Result<(), ErrorCode> {
        opts.apply(self)
    }

    #[inline(always)]
    ///Get property of the socket
    pub fn get_prop<T: Property<Self>>(&self) -> Result<T, ErrorCode> {
        T::get(self)
    }

    fn recv_inner<'a, const FLAGS: c_int>(&self, out: BufMut<'a>) -> Result<&'a [u8], ErrorCode> {
        let mut size = out.size;
        let result = unsafe {
            sys::nng_recv(**self, out.ptr as _, &mut size, FLAGS)
        };

        match result {
            0 => {
                let out = unsafe {
                    slice::from_raw_parts(out.ptr, size)
                };
                Ok(out)
            },
            code => Err(error(code)),
        }
    }

    #[inline(always)]
    ///Attempts to receive message, writing it in `out` buffer if it is of sufficient size,
    ///returning immediately if no message is available
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    ///
    ///Returns written bytes on success
    ///
    ///Returns [would block](https://docs.rs/error-code/3.2.0/error_code/struct.ErrorCode.html#method.is_would_block)
    ///error if no message is available.
    pub fn try_recv<'a>(&self, out: impl Into<BufMut<'a>>) -> Result<&'a [u8], ErrorCode> {
        self.recv_inner::<{sys::NNG_FLAG_NONBLOCK}>(out.into())
    }

    #[inline(always)]
    ///Receives message, writing it in `out` buffer if it is of sufficient size, waiting forever if none is available.
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    ///
    ///Returns written bytes on success
    pub fn recv<'a>(&self, out: impl Into<BufMut<'a>>) -> Result<&'a [u8], ErrorCode> {
        self.recv_inner::<0>(out.into())
    }

    ///Receives pending message, waiting forever if none is available.
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    fn recv_msg_inner<const FLAGS: c_int>(&self) -> Result<Message, ErrorCode> {
        let mut msg = ptr::null_mut();
        let result = unsafe {
            sys::nng_recvmsg(**self, &mut msg, FLAGS)
        };

        match ptr::NonNull::new(msg) {
            Some(ptr) => Ok(Message(ptr)),
            None => Err(error(result)),
        }
    }

    #[inline]
    ///Receives pending message, waiting forever if none is available.
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    pub fn recv_msg(&self) -> Result<Message, ErrorCode> {
        self.recv_msg_inner::<0>()
    }

    #[inline]
    ///Receives pending message, waiting forever if none is available.
    ///
    ///If underlying protocol doesn't support receiving messages, this shall return error always
    ///
    ///Returns None if no message is available.
    pub fn try_recv_msg(&self) -> Result<Option<Message>, ErrorCode> {
        match self.recv_msg_inner::<{sys::NNG_FLAG_NONBLOCK}>() {
            Ok(msg) => Ok(Some(msg)),
            Err(error) if error.is_would_block() => Ok(None),
            Err(error) => Err(error)
        }
    }

    #[inline]
    ///Creates new future that attempts to receive message from the socket.
    pub fn recv_msg_async(&self) -> Result<FutureResp, ErrorCode> {
        FutureResp::new(self)
    }

    #[inline]
    ///Encodes bytes into message and send it over the socket.
    ///
    ///Internally message shall be encoded and sent over
    pub fn send(&self, msg: Buf<'_>) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_send(**self, msg.ptr as _, msg.size, 0)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
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

    #[inline]
    ///Sends message over the socket asynchronously.
    ///
    ///If successful takes ownership of message.
    ///Otherwise returns message with error code.
    pub fn send_msg_async(&self, msg: Message) -> Result<FutureReq, ErrorCode> {
        FutureReq::new(self, msg)
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
    type Target = sys::nng_socket;

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

///Futures that resolves into message
pub struct FutureResp {
    aio: Aio,
}

impl FutureResp {
    ///Creates new future to retrieve message from the socket
    pub fn new(socket: &Socket) -> Result<Self, ErrorCode> {
        let aio = Aio::new()?;
        unsafe {
            sys::nng_recv_aio(**socket, aio.as_ptr())
        }

        Ok(Self {
            aio
        })
    }

    ///Sets future for cancelling
    pub fn cancel(&self) {
        unsafe {
            sys::nng_aio_cancel(self.aio.as_ptr())
        }
    }
}

impl Future for FutureResp {
    type Output = Result<Option<Message>, ErrorCode>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut this = self.as_mut();
        if this.aio.is_ready() {
            task::Poll::Ready(this.aio.get_msg())
        } else {
            this.aio.register_waker(ctx.waker());
            task::Poll::Pending
        }
    }
}

///Futures that awaits message to be sent
pub struct FutureReq {
    aio: Aio,
}

impl FutureReq {
    ///Creates new future taking ownership over `msg`
    pub fn new(socket: &Socket, msg: Message) -> Result<Self, ErrorCode> {
        let aio = Aio::new()?;
        unsafe {
            sys::nng_aio_set_msg(aio.as_ptr(), msg.as_ptr());
            sys::nng_send_aio(**socket, aio.as_ptr())
        }

        //AIO takes ownership of the message
        mem::forget(msg);

        Ok(Self {
            aio
        })
    }

    ///Sets future for cancelling
    pub fn cancel(&self) {
        unsafe {
            sys::nng_aio_cancel(self.aio.as_ptr())
        }
    }
}

impl Future for FutureReq {
    type Output = Result<(), (Message, ErrorCode)>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let mut this = self.as_mut();
        if this.aio.is_ready() {
            task::Poll::Ready(this.aio.get_send_result())
        } else {
            this.aio.register_waker(ctx.waker());
            task::Poll::Pending
        }
    }
}

///Socket listener
pub struct Listener(pub(crate) sys::nng_listener);

impl Listener {
    pub(crate) fn new(socket: &Socket, url: String<'_>) -> Result<Self, ErrorCode> {
        let url = url.as_ptr();
        let mut this = sys::nng_listener {
            id: 0
        };

        let result = unsafe {
            sys::nng_listener_create(&mut this, **socket, url as _)
        };

        match result {
            0 => Ok(Self(this)),
            code => Err(error(code))
        }
    }

    pub(crate) fn start(&self) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_listener_start(self.0, 0)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code))
        }
    }
}

impl Drop for Listener {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::nng_listener_close(self.0);
        }
    }
}

///Socket dialer
pub struct Dialer(pub(crate) sys::nng_dialer);

impl Dialer {
    pub(crate) fn new(socket: &Socket, url: String<'_>) -> Result<Self, ErrorCode> {
        let url = url.as_ptr();
        let mut this = sys::nng_dialer {
            id: 0
        };

        let result = unsafe {
            sys::nng_dialer_create(&mut this, **socket, url as _)
        };

        match result {
            0 => Ok(Self(this)),
            code => Err(error(code))
        }
    }

    pub(crate) fn start(&self, flags: c_int) -> Result<(), ErrorCode> {
        let result = unsafe {
            sys::nng_dialer_start(self.0, flags)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code))
        }
    }
}

impl Drop for Dialer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::nng_dialer_close(self.0);
        }
    }
}
