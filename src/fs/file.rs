use windows_sys::Win32::Foundation::ERROR_IO_PENDING;
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::{Foundation::HANDLE};


use std::io::{Result};


use std::ptr::null_mut;

use crate::context::IOType;

use crate::utils::{cvt, len};
use crate::{
    AsHandle, Context,
};

/// Addtional method for the `File` type.
pub trait FileExt: AsHandle {

    /// Execute an ovelapped read I/O on this file.
    /// 
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

    /// Execute an overlapped write I/O on this file.
    /// 
    fn _write(&self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        let len = len(&buff);
        let buff_ptr = buff.as_ptr();
        let handle = self.as_handle() as HANDLE;
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

    ///
    /// ```
    /// use iocp_rs::{CompletionPort, fs::FileExt, AsHandle};
    /// use std::io::Result;
    /// use std::path::Path;
    /// use std::fs::{OpenOptions, File};
    /// use std::collections::HashMap;
    /// use std::os::windows::io::AsRawHandle;
    /// use std::os::windows::fs::OpenOptionsExt;
    /// use windows_sys::Win32::{Storage::FileSystem::FILE_FLAG_OVERLAPPED, Foundation::HANDLE};
    /// 
    /// struct MyFile {
    ///     inner: File
    /// }
    /// 
    /// impl AsHandle for MyFile {
    ///     type Handle = HANDLE;
    ///     fn as_handle(&self) -> Self::Handle {
    ///         self.inner.as_raw_handle() as HANDLE
    ///     }
    /// }
    /// 
    /// impl FileExt for MyFile {}
    /// 
    /// fn main() -> Result<()> {
    ///     let cmp = CompletionPort::new(1)?;
    ///     let mut file = OpenOptions::new()
    ///                         .read(true)
    ///                         .custom_flags(FILE_FLAG_OVERLAPPED)
    ///                         .open("./tmp.txt")?;
    /// 
    ///     let mut file = MyFile {inner: file};
    ///     cmp.add(1, &file)?;
    ///     let mut buff = vec![0; 10];
    ///     
    ///     let context = FileExt::read(&mut file, buff)?;
    ///     let mut map = HashMap::new();
    ///     map.insert(1, context);
    ///
    ///     let mut result_list = cmp.get_many(map.len(), None)?;
    ///     let result = result_list.remove(0);
    ///     let context = map.remove(&result.token()).unwrap();
    ///     assert_eq!(&context.get_buff()[..result.bytes_used() as usize], b"123sdf");
    ///     assert_eq!(result.bytes_used() as usize, 6usize);
    ///     Ok(())
    /// }
    /// ```
    fn read(&mut self, buff: Vec<u8>) -> Result<Context> {
        self._read(buff, 0)
    }

    fn read_at(&mut self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        self._read(buff, offset)
    }

    /// 
    /// ```
    /// use iocp_rs::{CompletionPort, fs::FileExt, AsHandle};
    /// use std::io::Result;
    /// use std::path::Path;
    /// use std::collections::HashMap;
    /// use std::fs::{OpenOptions, File};
    /// use std::os::windows::io::AsRawHandle;
    /// use std::os::windows::fs::OpenOptionsExt;
    /// use windows_sys::Win32::{Storage::FileSystem::FILE_FLAG_OVERLAPPED, Foundation::HANDLE};
    ///
    /// struct MyFile {
    ///     inner: File
    /// }
    /// 
    /// impl AsHandle for MyFile {
    ///     type Handle = HANDLE;
    ///     fn as_handle(&self) -> Self::Handle {
    ///         self.inner.as_raw_handle() as HANDLE
    ///     }
    /// }
    /// 
    /// impl FileExt for MyFile {}
    /// 
    /// fn main() -> Result<()> {
    ///     let cmp = CompletionPort::new(1)?;
    ///     let mut file = OpenOptions::new()
    ///                         .write(true)
    ///                         .custom_flags(FILE_FLAG_OVERLAPPED)
    ///                         .open("./tmp.txt")?;
    /// 
    ///     let mut file = MyFile {inner: file};
    ///     cmp.add(1, &file)?;
    ///     let mut buff = b"123".to_vec();
    ///     
    ///     let context = file.write(buff)?;
    ///     let mut map = HashMap::new();
    ///     map.insert(1, context);
    ///
    ///     let mut result_list = cmp.get_many(map.len(), None)?;
    ///     for result in result_list {
    ///         if map.contains_key(&result.token()) {
    ///             dbg!(result.bytes_used());
    ///         }   
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn write(&self, buff: Vec<u8>) -> Result<Context> {
        self._write(buff, 0)
    }

    fn write_at(&self, buff: Vec<u8>, offset: u64) -> Result<Context> {
        self._write(buff, offset)
    }
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;

    use crate::{
        fs::FileExt,
        AsHandle, CompletionPort,
    };
    use std::{
        fs::{File, OpenOptions},
        os::windows::prelude::{OpenOptionsExt, AsRawHandle},
    };

    impl AsHandle for File {

        fn as_handle(&self) -> HANDLE {
            self.as_raw_handle() as HANDLE
        }
    }

    impl FileExt for File {}

    #[test]
    fn read_file() {
        let cmp = CompletionPort::new(1).unwrap();
        let mut file = OpenOptions::new()
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .read(true)
            .open("..\\test.txt")
            .unwrap();
        cmp.add(1, &file).unwrap();
        let buff = vec![0; 10];

        let context = FileExt::read(&mut file, buff).unwrap();

        let result = cmp.get(None).unwrap();
        assert_eq!(
            &context.get_buff()[..result.bytes_used() as usize],
            b"123".as_slice()
        );
    }

    #[test]
    fn write_file() {
        let cmp = CompletionPort::new(1).unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
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
