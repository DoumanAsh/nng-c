use core::{ops, ptr, slice, mem, fmt};

use crate::error::{ErrorCode, error};

use nng_c_sys::nng_msg;
use nng_c_sys::{nng_msg_alloc, nng_msg_free, nng_msg_capacity, nng_msg_reserve};
use nng_c_sys::{nng_msg_clear, nng_msg_dup};
use nng_c_sys::{nng_msg_body, nng_msg_len};
use nng_c_sys::{nng_msg_trim, nng_msg_chop};
use nng_c_sys::{nng_msg_chop_u16, nng_msg_chop_u32, nng_msg_chop_u64};
use nng_c_sys::{nng_msg_trim_u16, nng_msg_trim_u32, nng_msg_trim_u64};
use nng_c_sys::{nng_msg_append, nng_msg_append_u16, nng_msg_append_u32, nng_msg_append_u64};
use nng_c_sys::{nng_msg_insert, nng_msg_insert_u16, nng_msg_insert_u32, nng_msg_insert_u64};
use nng_c_sys::{nng_msg_header, nng_msg_header_len};

///Message primitive
pub struct Message(pub(crate) ptr::NonNull<nng_msg>);

impl Message {
    #[inline(always)]
    ///Creates empty message
    ///
    ///Returns `None` if unable to allocate memory
    pub fn new() -> Option<Self> {
        Self::with_capaicty(0)
    }

    #[inline]
    ///Creates message with pre-allocated buffer of provided `size`
    ///
    ///Returns `None` if unable to allocate memory
    pub fn with_capaicty(size: usize) -> Option<Self> {
        let mut ptr = ptr::null_mut();
        unsafe {
            nng_msg_alloc(&mut ptr, size);
        }

        ptr::NonNull::new(ptr).map(Self)
    }

    #[inline]
    ///Creates new copy of the message.
    ///
    ///Returns `None` if unable to allocate memory
    pub fn dup(&self) -> Option<Self> {
        let mut ptr = ptr::null_mut();
        unsafe {
            nng_msg_dup(&mut ptr, self.0.as_ptr());
        }

        ptr::NonNull::new(ptr).map(Self)
    }

    #[inline(always)]
    ///Reserves additional space to accommodate specified `capacity`
    ///
    ///Does nothing, if message already has enough space.
    ///
    ///Returns error only if reallocation failed
    pub fn reserve(&mut self, capacity: usize) -> Result<(), ErrorCode> {
        let result = unsafe {
            nng_msg_reserve(self.0.as_ptr(), capacity)
        };

        match result {
            0 => Ok(()),
            code => Err(error(code))
        }
    }

    #[inline(always)]
    ///Returns length of message body
    pub fn len(&self) -> usize {
        unsafe {
            nng_msg_len(self.0.as_ptr())
        }
    }

    #[inline(always)]
    ///Returns capacity of message body
    pub fn capaciy(&self) -> usize {
        unsafe {
            nng_msg_capacity(self.0.as_ptr())
        }
    }

    #[inline(always)]
    ///Clears content of the message.
    ///
    ///Note that it is primarily sets length of body to 0.
    ///Allocated capacity remains the same.
    pub fn clear(&mut self) {
        unsafe {
            nng_msg_clear(self.0.as_ptr())
        }
    }

    #[inline(always)]
    ///Shortens body length, keeping `len` starting elements
    ///
    ///Has no effect if `len` is equal or greater to current body's length
    pub fn truncate(&mut self, len: usize) {
        let size = self.len().saturating_sub(len);
        unsafe {
            nng_msg_chop(self.0.as_ptr(), size);
        }
    }

    #[inline(always)]
    ///Shortens body length, keeping `len` last elements inside
    ///
    ///Has no effect if `len` is equal or greater to current body's length
    pub fn truncate_start(&mut self, len: usize) {
        let size = self.len().saturating_sub(len);
        unsafe {
            nng_msg_trim(self.0.as_ptr(), size);
        }
    }

    #[inline(always)]
    ///Returns reference to the body content
    pub fn body(&self) -> &[u8] {
        let ptr = self.0.as_ptr();
        unsafe {
            let body = nng_msg_body(ptr);
            let len = nng_msg_len(ptr);
            slice::from_raw_parts(body as *const u8, len)
        }
    }

    #[inline(always)]
    ///Returns reference to the header content
    pub fn header(&self) -> &[u8] {
        let ptr = self.0.as_ptr();
        unsafe {
            let body = nng_msg_header(ptr);
            let len = nng_msg_header_len(ptr);
            slice::from_raw_parts(body as *const u8, len)
        }
    }

    fn push_inner<T: Copy>(&mut self, value: T, insertor: unsafe extern "C" fn(*mut nng_msg, T) -> core::ffi::c_int) -> Result<(), ErrorCode> {
        let result = unsafe {
            (insertor)(self.0.as_ptr(), value)
        };
        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    fn pop_inner<T: Copy>(&mut self, extractor: unsafe extern "C" fn(*mut nng_msg, *mut T) -> core::ffi::c_int) -> Option<T> {
        let mut out = mem::MaybeUninit::uninit();
        let result = unsafe {
            (extractor)(self.0.as_ptr(), out.as_mut_ptr())
        };
        match result {
            0 => Some(unsafe {
                out.assume_init()
            }),
            _ => None
        }
    }

    //pop
    #[inline(always)]
    ///Extracts u16 from the end of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_u16(&mut self) -> Option<u16> {
        self.pop_inner(nng_msg_chop_u16)
    }

    #[inline(always)]
    ///Extracts u32 from the end of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_u32(&mut self) -> Option<u32> {
        self.pop_inner(nng_msg_chop_u32)
    }

    #[inline(always)]
    ///Extracts u64 from the end of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_u64(&mut self) -> Option<u64> {
        self.pop_inner(nng_msg_chop_u64)
    }

    #[inline(always)]
    ///Extracts u16 from the start of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_front_u16(&mut self) -> Option<u16> {
        self.pop_inner(nng_msg_trim_u16)
    }

    #[inline(always)]
    ///Extracts u32 from the start of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_front_u32(&mut self) -> Option<u32> {
        self.pop_inner(nng_msg_trim_u32)
    }

    #[inline(always)]
    ///Extracts u64 from the start of body, encoding it into native byte order
    ///
    ///Returns `None` if there is not enough space
    pub fn pop_front_u64(&mut self) -> Option<u64> {
        self.pop_inner(nng_msg_trim_u64)
    }

    //push
    #[inline(always)]
    ///Appends u16 to the end of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn append_u16(&mut self, value: u16) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_append_u16)
    }

    #[inline(always)]
    ///Appends u32 to the end of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn append_u32(&mut self, value: u32) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_append_u32)
    }

    #[inline(always)]
    ///Appends u64 to the end of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn append_u64(&mut self, value: u64) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_append_u64)
    }

    ///Appends `bytes` to the message body.
    pub fn append(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        let result = unsafe {
            nng_msg_append(self.0.as_ptr(), bytes.as_ptr() as _, bytes.len())
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }

    #[inline(always)]
    ///Inserts u16 at the start of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn insert_u16(&mut self, value: u16) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_insert_u16)
    }

    #[inline(always)]
    ///Inserts u32 at the start of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn insert_u32(&mut self, value: u32) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_insert_u32)
    }

    #[inline(always)]
    ///Inserts u64 at the start of body, encoding it into network byte order
    ///
    ///Returns `Err` if there is not enough space
    pub fn insert_u64(&mut self, value: u64) -> Result<(), ErrorCode> {
        self.push_inner(value, nng_msg_insert_u64)
    }

    ///Inserts `bytes` at the start of the body.
    pub fn insert(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        let result = unsafe {
            nng_msg_insert(self.0.as_ptr(), bytes.as_ptr() as _, bytes.len())
        };

        match result {
            0 => Ok(()),
            code => Err(error(code)),
        }
    }
}

impl Clone for Message {
    #[inline]
    fn clone(&self) -> Self {
        self.dup().unwrap()
    }
}

impl ops::Deref for Message {
    type Target = ptr::NonNull<nng_msg>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for Message {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for Message {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Message").field("len", &self.len()).finish()
    }
}

impl Drop for Message {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            nng_msg_free(self.0.as_ptr())
        }
    }
}
