use crate::{
    cvt,
    utils::{dur_to_ms, len},
    AsHandle, OperationalResult,
};
use std::{
    io::{Error, Result},
    mem::zeroed,
    ptr::null_mut,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::IO::{
        CreateIoCompletionPort, GetQueuedCompletionStatus, GetQueuedCompletionStatusEx,
        PostQueuedCompletionStatus, OVERLAPPED_ENTRY,
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

    /// Register a handle and token with CompletionPort.
    /// The same token and handle cannot be registered in CompletionPort again unless the handle has been closed.
    /// 
    /// ```
    /// 
    /// ```
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

    /// Get many result by Context lists, and return OperationalResult lists.
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
                .map(OperationalResult::new)
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

unsafe impl Send for CompletionPort {}
unsafe impl Sync for CompletionPort {}

#[cfg(test)]
mod tests {
    use std::{fs::OpenOptions, os::windows::prelude::OpenOptionsExt};

    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;

    use crate::{CompletionPort, fs::FileExt};

    #[test]
    fn repeat_add() {
        let mut cmp = CompletionPort::new(1).unwrap();

        let mut file = OpenOptions::new()
                                            .custom_flags(FILE_FLAG_OVERLAPPED)
                                            .read(true)
                                            .open("..\\test.txt")
                                            .unwrap();

        cmp.add(1, &file).unwrap();

        let context_1 = file.read(vec![0; 3]).unwrap();

        let result_list = cmp.get_many(2, None).unwrap();
        for result in result_list {
            dbg!(result.token());
        }

        drop(file);
        
        let mut file = OpenOptions::new()
                                            .custom_flags(FILE_FLAG_OVERLAPPED)
                                            .read(true)
                                            .open("..\\test.txt")
                                            .unwrap();

        cmp.add(1, &file).unwrap();

        // cmp.add(2, &file).unwrap();
        let context_2 = file.read(vec![0; 2]).unwrap();

        let result_list = cmp.get_many(2, None).unwrap();
        for result in result_list {
            dbg!(result.token());
        }


        drop(file);
        drop(cmp);
    }
}