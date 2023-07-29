use std::{mem::zeroed, ptr::null_mut};

use windows_sys::Win32::System::IO::{OVERLAPPED, OVERLAPPED_ENTRY};

pub struct OperationalResult {
    entry: OVERLAPPED_ENTRY,
}

impl OperationalResult {
    pub(crate) fn from_entry(entry: OVERLAPPED_ENTRY) -> Self {
        Self { entry }
    }

    pub fn new(token: usize, bytes_used: u32, over_lapped: *mut OVERLAPPED) -> Self {
        Self {
            entry: OVERLAPPED_ENTRY {
                lpCompletionKey: token,
                lpOverlapped: over_lapped,
                Internal: 0,
                dwNumberOfBytesTransferred: bytes_used,
            },
        }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, null_mut())
    }

    pub fn token(&self) -> usize {
        self.entry.lpCompletionKey
    }

    pub fn offset(&self) -> u64 {
        let over_lapped = unsafe { &(*self.entry.lpOverlapped) };

        let low_offset = unsafe { over_lapped.Anonymous.Anonymous.Offset as u64 };
        let high_offset = unsafe { over_lapped.Anonymous.Anonymous.OffsetHigh as u64 };

        (high_offset << 32) | low_offset
    }

    pub fn bytes_used(&self) -> u32 {
        self.entry.dwNumberOfBytesTransferred
    }

    pub(crate) fn over_lapped_ptr(&self) -> *mut OVERLAPPED {
        self.entry.lpOverlapped
    }
}
