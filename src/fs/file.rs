use windows_sys::Win32::Foundation::ERROR_IO_PENDING;
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::FILE_FLAG_OVERLAPPED};

use std::fs::{File as StdFile, Metadata, OpenOptions as StdOpenOptions, Permissions};
use std::io::{Error, Result};
use std::os::windows::prelude::{AsRawHandle, OpenOptionsExt};
use std::path::Path;
use std::ptr::null_mut;

use crate::context::IOType;
use crate::fs::OpenOptions;
use crate::io::Read;
use crate::utils::{cvt, len};
use crate::{
    io::{ReadAt, Write, WriteAt},
    AsHandle, Context,
};

pub struct File {
    pub(crate) inner: StdFile,
}

impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(path)
    }

    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(path)
    }

    pub fn metadata(&self) -> Result<Metadata> {
        self.inner.metadata()
    }

    pub fn set_len(&self, size: u64) -> Result<()> {
        self.inner.set_len(size)
    }

    pub fn set_permissions(&self, perm: Permissions) -> Result<()> {
        self.inner.set_permissions(perm)
    }

    fn _read(&mut self, mut buff: Vec<u8>, offset: u64) -> Result<Context> {
        let len = len(&buff);
        let buff_ptr = buff.as_mut_ptr();
        let handle = self.as_handle();
        let mut context = Context::new(handle, buff, IOType::Read);
        let over_lapped_ptr = context.over_lapped_ptr();
        context.set_offset(offset);

        let ret = unsafe { ReadFile(handle, buff_ptr as *mut _, len, null_mut(), over_lapped_ptr) };

        match cvt(ret) {
            Ok(_) => Ok(context),
            Err(e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(context),
            Err(e) => Err(e),
        }
    }

    fn _write(&self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        let len = len(&buff);
        let buff_ptr = buff.as_ptr();
        let handle = self.inner.as_raw_handle() as HANDLE;
        let mut context = Context::new(handle, buff, IOType::Write);
        let over_lapped_ptr = context.over_lapped_ptr();
        context.set_offset(offset);

        let ret = unsafe { WriteFile(handle, buff_ptr, len, null_mut(), over_lapped_ptr) };

        match cvt(ret) {
            Ok(_) => Ok(context),
            Err(e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

impl AsHandle for File {
    type Handle = HANDLE;

    fn as_handle(&self) -> Self::Handle {
        self.inner.as_raw_handle() as HANDLE
    }
}

impl Read for File {
    ///
    /// ```
    /// use iocp_rs::{CompletionPort, fs::{File, OpenOptions}, io::Read};
    /// use std::io::Result;
    /// use std::path::Path;
    ///
    /// fn main() -> Result<()> {
    ///     let cmp = CompletionPort::new(1)?;
    ///     let mut file = OpenOptions::new().read(true).create_new(true).open("./tmp.txt")?;
    ///     cmp.add(1, &file)?;
    ///     let mut buff = vec![0; 10];
    ///     
    ///     let context = file.read(buff)?;
    ///     let mut list = vec![context];
    ///
    ///     let mut result_list = cmp.get_many(&mut list, None)?;
    ///     let (buff, size, _io_type) = result_list.remove(0).get();
    ///     assert_eq!(&buff[..size], b"123sdf");
    ///     assert_eq!(&size, &6usize);
    ///     Ok(())
    /// }
    /// ```
    fn read(&mut self, buff: Vec<u8>) -> Result<Context> {
        self._read(buff, 0)
    }
}

impl ReadAt for File {
    fn read_at(&mut self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        self._read(buff, offset)
    }
}

impl Write for File {
    ///
    /// ```
    /// use iocp_rs::{CompletionPort, fs::{File, OpenOptions}, io::Write};
    /// use std::io::Result;
    /// use std::path::Path;
    ///
    /// fn main() -> Result<()> {
    ///     let cmp = CompletionPort::new(1)?;
    ///     let mut file = OpenOptions::new().read(true).create_new(true).open("./tmp.txt")?;
    ///     cmp.add(1, &file)?;
    ///     let mut buff = b"123".to_vec();
    ///     
    ///     let context = file.write(buff)?;
    ///     let mut list = vec![context];
    ///
    ///     let mut result_list = cmp.get_many(&mut list, None)?;
    ///     let (buff, size, _io_type) = result_list.remove(0).get();
    ///     Ok(())
    /// }
    /// ```
    fn write(&self, buff: Vec<u8>) -> Result<Context> {
        self._write(buff, 0)
    }
}

impl WriteAt for File {
    fn write_at(&self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        self._write(buff, offset)
    }
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;

    use crate::{
        fs::OpenOptions,
        io::{Read, Write},
        CompletionPort,
    };

    #[test]
    fn read_file() {
        let cmp = CompletionPort::new(1).unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .open("..\\test.txt")
            .unwrap();
        cmp.add(1, &file).unwrap();
        let buff = vec![0; 10];

        let context = file.read(buff).unwrap();

        let mut result = cmp.get(None).unwrap();
        assert_eq!(&context.get_buff()[..result.bytes_used() as usize], b"123".as_slice());
    }

    #[test]
    fn write_file() {
        let cmp = CompletionPort::new(1).unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("..\\test.txt")
            .unwrap();
        cmp.add(1, &file).unwrap();
        let buff = b"123".to_vec();

        let context = file.write(buff).unwrap();

        let result = cmp.get(None).unwrap();
        assert_eq!(&context.get_buff()[..result.bytes_used() as usize], b"123");
        // let (mut result_list, list) = cmp.get_many(list, None).unwrap();
        // let (buff, size, _io_type) = result_list.remove(0).get();
        // assert_eq!(&buff, b"123");
        // assert_eq!(size, 3);
    }
}
