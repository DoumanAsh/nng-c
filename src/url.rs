//! Url wrapper
use core::{fmt, mem};

use alloc::vec::Vec;

///Url's static buffer size
pub const STATIC_SIZE: usize = mem::size_of::<usize>() * 2;

enum State<'a> {
    //This variant should already include zero char at the end
    Slice(&'a [u8]),
    //Static buffer, if slice fits it with zero char
    Static([u8; STATIC_SIZE]),
    //Last resort
    Heap(Vec<u8>),
}

const _: () = {
    assert!(mem::size_of::<State>() == 24);
};

#[repr(transparent)]
///Wrapper for NNG Url (C String)
pub struct Url<'a> {
    state: State<'a>
}

impl<'a> Url<'a> {
    ///Creates new url.
    ///
    ///If `url` ends with null character, then it slice will be used as it is otherwise
    ///it shall create buffer to store `url` with null terminating character appended
    pub fn new(url: &'a [u8]) -> Self {
        debug_assert_ne!(url.len(), 0);
        let state = if url.ends_with(&[0]) {
            State::Slice(url)
        } else if url.len() < STATIC_SIZE {
            let mut buffer = [0u8; STATIC_SIZE];
            buffer[..url.len()].copy_from_slice(url);
            buffer[url.len()] = 0;
            State::Static(buffer)
        } else {
            let mut buffer = Vec::with_capacity(url.len().saturating_add(1));
            buffer.extend_from_slice(url);
            buffer.push(0);
            State::Heap(buffer)
        };

        Self {
            state
        }
    }

    ///Returns pointer to the underlying buffer
    pub fn as_ptr(&self) -> *const u8 {
        match &self.state {
            State::Heap(buf) => buf.as_ptr(),
            State::Static(buf) => buf.as_ptr() as _,
            State::Slice(buf) => buf.as_ptr(),
        }
    }

    ///Returns URL without null termination character
    pub fn as_bytes(&self) -> &[u8] {
        match &self.state {
            State::Heap(buf) => &buf[..buf.len()-1],
            State::Static(buf) => {
                let mut cursor = 0;
                while cursor < STATIC_SIZE {
                    if buf[cursor] == 0 {
                        break;
                    }
                    cursor = cursor.saturating_add(1);
                }
                &buf[..cursor]
            },
            State::Slice(buf) => &buf[..buf.len()-1],
        }
    }
}

impl fmt::Debug for Url<'_> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_bytes(), fmt)
    }
}

impl PartialEq<[u8]> for Url<'_> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        PartialEq::eq(self.as_bytes(), other)
    }
}

impl PartialEq<&[u8]> for Url<'_> {
    #[inline(always)]
    fn eq(&self, other: &&[u8]) -> bool {
        PartialEq::eq(self.as_bytes(), *other)
    }
}

impl PartialEq<str> for Url<'_> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_bytes(), other.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for Url<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a str> for Url<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::new(value.as_bytes())
    }
}
