use ascii::{AsciiChar, AsciiStr, AsciiString};
use core::mem;

struct AsciiStringLayout {
    vec: Vec<AsciiChar>,
}

impl AsciiStringLayout {
    pub const fn new() -> AsciiString {
        unsafe { mem::transmute(AsciiStringLayout { vec: Vec::new() }) }
    }
}

struct Repr {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
}

impl Repr {
    pub unsafe const fn from_static_bytes(bytes: &'static [u8]) -> Self {
        Self {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            cap: 0,
        }
    }

    pub unsafe const fn from_static_str(bytes: &'static str) -> Self {
        Self::from_static_bytes(string.as_bytes())
    }
}

pub struct Id {
    inner: Repr,
}

impl Id {
    pub const fn new() -> Self {
        Self {
            inner: AsciiStringLayout::new(),
        }
    }
}
