//! C String wrapper
use core::{fmt, mem};

use alloc::vec::Vec;

///String's static buffer size
pub const STATIC_SIZE: usize = mem::size_of::<usize>() * 2;

#[derive(Clone)]
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
#[derive(Clone)]
///Wrapper for C string
pub struct String<'a> {
    state: State<'a>
}

impl<'a> String<'a> {
    #[inline]
    ///Creates new instance from C string (null terminated)
    ///
    ///Returns None if input has no NULL character at the end
    pub const fn try_new_c(string: &'a [u8]) -> Option<Self> {
        if string[string.len() - 1] == 0 {
            Some(Self {
                state: State::Slice(string)
            })
        } else {
            None
        }
    }

    #[inline]
    ///Creates new instance from C string (null terminated)
    ///
    ///Panics if input has no NULL character at the end
    pub const fn new_c(string: &'a [u8]) -> Self {
        if string[string.len() - 1] == 0 {
            Self {
                state: State::Slice(string)
            }
        } else {
            panic!("string is not NULL terminated")
        }
    }

    ///Creates new String.
    ///
    ///If `string` ends with null character, then it slice will be used as it is otherwise
    ///it shall create buffer to store `string` with null terminating character appended
    pub fn new(string: &'a [u8]) -> Self {
        let state = if let Some(this) = Self::try_new_c(string) {
            this.state
        } else if string.len() < STATIC_SIZE {
            let mut buffer = [0u8; STATIC_SIZE];
            buffer[..string.len()].copy_from_slice(string);
            buffer[string.len()] = 0;
            State::Static(buffer)
        } else {
            let mut buffer = Vec::with_capacity(string.len().saturating_add(1));
            buffer.extend_from_slice(string);
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

    ///Returns String without null termination character
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

impl fmt::Debug for String<'_> {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_bytes(), fmt)
    }
}

impl PartialEq<[u8]> for String<'_> {
    #[inline(always)]
    fn eq(&self, other: &[u8]) -> bool {
        PartialEq::eq(self.as_bytes(), other)
    }
}

impl PartialEq<&[u8]> for String<'_> {
    #[inline(always)]
    fn eq(&self, other: &&[u8]) -> bool {
        PartialEq::eq(self.as_bytes(), *other)
    }
}

impl PartialEq<str> for String<'_> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_bytes(), other.as_bytes())
    }
}

impl<'a> From<&'a [u8]> for String<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a str> for String<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::new(value.as_bytes())
    }
}
