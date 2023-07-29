use crate::{
    context::Context,
    cvt,
    utils::{dur_to_ms, len},
    AsHandle, OperationalResult,
};
use std::{
    collections::VecDeque,
    io::{Error, Result},
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
    fn new(num_threads: u32) -> Result<Self> {
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

    /// Get a result from CompletionPort while the task is finished.
    ///
    pub fn get(&self, timeout: Option<Duration>) -> Result<OperationalResult> {
        let timeout = dur_to_ms(timeout);
        let mut bytes_used = 0;
        let mut token = 0;
        let mut over_lapped = null_mut::<OVERLAPPED>();

        let ret = unsafe {
            GetQueuedCompletionStatus(
                self.handle,
                &mut bytes_used,
                &mut token,
                &mut over_lapped as *mut _,
                timeout,
            )
        };

        cvt(ret).map(|_| OperationalResult::new(token, bytes_used, over_lapped))
    }

    /// Get many result by Context lists, and return OperationalResult lists.
    pub fn get_many(
        &self,
        list: &mut Vec<Context>,
        timeout: Option<Duration>,
    ) -> Result<Vec<OperationalResult>> {
        let mut entries = list
            .iter()
            .map(Context::over_lapped_ptr)
            .map(|over_lapped_ptr| OVERLAPPED_ENTRY {
                lpCompletionKey: 0,
                lpOverlapped: over_lapped_ptr,
                Internal: 0,
                dwNumberOfBytesTransferred: 0,
            })
            .collect::<Vec<OVERLAPPED_ENTRY>>();
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
            list.drain(..removed).for_each(|_| {});
            Ok(entries
                .drain(..removed)
                .map(OperationalResult::from_entry)
                .collect::<_>())
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

pub struct CompletionPortGraud {
    completion_port: CompletionPort,
}

impl CompletionPortGraud {
    pub fn new(num_threads: u32) -> Result<Self> {
        CompletionPort::new(num_threads).map(|completion_port| Self { completion_port })
    }

    pub fn run<F: FnOnce(&mut CompletionPort) + 'static>(mut self, func: F) -> Result<()> {
        func(&mut self.completion_port);

        let ret = unsafe { CloseHandle(self.completion_port.handle) };

        cvt(ret).map(|_| ())
    }
}
