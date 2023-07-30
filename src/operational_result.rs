use std::{mem::zeroed, ptr::null_mut};

use windows_sys::Win32::System::IO::{OVERLAPPED, OVERLAPPED_ENTRY};

use crate::{context::IOType, Context};

pub struct OperationalResult {
    buff: Vec<u8>,
    io_type: IOType,
    over_lapped: OVERLAPPED,
    entry: OVERLAPPED_ENTRY,
}

impl OperationalResult {
    pub fn new(context: Context, entry: OVERLAPPED_ENTRY) -> Self {
        Self {
            buff: context.buff,
            io_type: context.io_type,
            over_lapped: context.over_lapped,
            entry,
        }
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

    pub fn bytes_used(&self) -> usize {
        self.entry.dwNumberOfBytesTransferred as usize
    }

    pub(crate) fn over_lapped_ptr(&self) -> *mut OVERLAPPED {
        self.entry.lpOverlapped
    }

    pub fn get(self) -> (Vec<u8>, usize, IOType) {
        (self.buff, self.bytes_used(), self.io_type)
    }
}
