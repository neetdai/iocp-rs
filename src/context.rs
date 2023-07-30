use std::{
    io::{Error, Result},
    mem::zeroed,
};

use windows_sys::Win32::{
    Foundation::{FALSE, HANDLE},
    System::IO::{CancelIoEx, OVERLAPPED},
};

pub enum IOType {
    Read,
    Write,
}

pub struct Context {
    pub(crate) buff: Vec<u8>,
    handle: HANDLE,
    pub(crate) io_type: IOType,
    pub(crate) over_lapped: OVERLAPPED,
}

impl Context {
    pub(crate) fn new(handle: HANDLE, buff: Vec<u8>, io_type: IOType) -> Self {
        let over_lapped = unsafe { zeroed::<OVERLAPPED>() };
        Self {
            handle,
            buff,
            io_type,
            over_lapped,
        }
    }

    pub(crate) fn set_offset(&mut self, offset: u64) {
        let low_offset = (offset & (u32::MAX as u64)) as u32;
        let high_offset = (offset >> 32) as u32;

        self.over_lapped.Anonymous.Anonymous.Offset = low_offset;
        self.over_lapped.Anonymous.Anonymous.OffsetHigh = high_offset;
    }

    pub(crate) fn offset(&self) -> u64 {
        let low_offset = unsafe { self.over_lapped.Anonymous.Anonymous.Offset as u64 };
        let high_offset = unsafe { self.over_lapped.Anonymous.Anonymous.OffsetHigh as u64 };

        (high_offset << 32) | low_offset
    }

    pub(crate) fn over_lapped_ptr(&mut self) -> *mut OVERLAPPED {
        (&mut self.over_lapped) as *mut _
    }

    pub fn cancel(self) -> Result<()> {
        let ret = unsafe { CancelIoEx(self.handle, &self.over_lapped as *const _) };

        if ret == FALSE {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
