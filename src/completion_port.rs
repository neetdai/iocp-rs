use crate::{
    context::Context,
    cvt,
    utils::{dur_to_ms, len},
    AsHandle, OperationalResult,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{Error, Result},
    mem::zeroed,
    ptr::null_mut,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::IO::{
        CreateIoCompletionPort, GetQueuedCompletionStatus, GetQueuedCompletionStatusEx,
        PostQueuedCompletionStatus, OVERLAPPED, OVERLAPPED_ENTRY,
    },
};

pub struct CompletionPort {
    handle: HANDLE,
}

impl CompletionPort {
    /// Create a CompletionPort with specify then concurrent
    pub fn new(num_threads: u32) -> Result<Self> {
        let ret = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, 0, 0, num_threads) };

        if ret == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(Self { handle: ret })
        }
    }

    ///
    pub fn add<A: AsHandle<Handle = HANDLE>>(&self, token: usize, io_object: &A) -> Result<()> {
        let handle = io_object.as_handle();

        let ret = unsafe { CreateIoCompletionPort(handle, self.handle, token, 0) };

        if ret == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn get(&self, timeout: Option<Duration>) -> Result<OperationalResult> {
        let mut ptr = null_mut();
        let mut bytes_used = 0;
        let mut token = 0;
        let timeout = dur_to_ms(timeout);

        let ret = unsafe {
            GetQueuedCompletionStatus(self.handle, &mut bytes_used, &mut token, &mut ptr, timeout)
        };

        if ret == 0 {
            Err(Error::last_os_error())
        } else {
            let entry = OVERLAPPED_ENTRY {
                lpCompletionKey: token,
                Internal: 0,
                lpOverlapped: ptr,
                dwNumberOfBytesTransferred: bytes_used,
            };

            Ok(OperationalResult::new(entry))
        }
    }

    // /// Get many result by Context lists, and return OperationalResult lists.
    pub fn get_many(
        &self,
        size: usize,
        timeout: Option<Duration>,
    ) -> Result<Vec<OperationalResult>> {
        let mut entries = vec![unsafe { zeroed::<OVERLAPPED_ENTRY>() }; size];

        let mut removed = 0;
        let timeout = dur_to_ms(timeout);
        let len = len(&entries);

        let ret = unsafe {
            GetQueuedCompletionStatusEx(
                self.handle,
                entries.as_mut_ptr(),
                len,
                &mut removed,
                timeout,
                0,
            )
        };

        if ret == 0 {
            Err(Error::last_os_error())
        } else {
            let removed = removed as usize;

            unsafe {
                entries.set_len(removed);
            }

            Ok(entries
                .drain(..removed)
                .map(|entry| OperationalResult::new(entry))
                .collect())
        }
    }

    pub fn post(&self, result: OperationalResult) -> Result<()> {
        let ret = unsafe {
            PostQueuedCompletionStatus(
                self.handle,
                result.bytes_used(),
                result.token(),
                result.over_lapped_ptr(),
            )
        };

        cvt(ret).map(|_| ())
    }
}

impl Drop for CompletionPort {
    fn drop(&mut self) {
        let ret = unsafe { CloseHandle(self.handle) };
        if ret == 0 {
            panic!("error {:?}", Error::last_os_error());
        }
    }
}
