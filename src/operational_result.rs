

use windows_sys::Win32::System::IO::{OVERLAPPED, OVERLAPPED_ENTRY};

pub struct OperationalResult {
    entry: OVERLAPPED_ENTRY,
}

impl OperationalResult {
    pub fn new(entry: OVERLAPPED_ENTRY) -> Self {
        Self { entry }
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
