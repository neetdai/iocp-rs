use windows_sys::Win32::Foundation::ERROR_IO_PENDING;
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, ReadFile};
use windows_sys::Win32::System::IO::CancelIo;
use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::FILE_FLAG_OVERLAPPED};

use std::fs::{File as StdFile, Metadata, OpenOptions as StdOpenOptions, Permissions};
use std::io::{Error, Result};
use std::os::windows::prelude::{AsRawHandle, OpenOptionsExt};
use std::path::Path;
use std::ptr::null_mut;

use crate::context::IOType;
use crate::io::Read;
use crate::utils::{cvt, len};
use crate::{AsHandle, Context};

pub struct OpenOptions {
    opt: StdOpenOptions,
}

impl OpenOptions {
    pub fn new() -> Self {
        let mut opt = StdOpenOptions::new();
        opt.custom_flags(FILE_FLAG_OVERLAPPED);

        Self { opt }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.opt.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.opt.write(write);
        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.opt.append(append);
        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.opt.create(create);
        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.opt.create_new(create_new);
        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.opt.truncate(truncate);
        self
    }

    pub fn access_mode(&mut self, access: u32) -> &mut Self {
        self.opt.access_mode(access);
        self
    }

    pub fn share_mode(&mut self, val: u32) -> &mut Self {
        self.opt.share_mode(val);
        self
    }

    pub fn attributes(&mut self, val: u32) -> &mut Self {
        self.opt.attributes(val);
        self
    }

    pub fn custom_flags(&mut self, flags: u32) -> &mut Self {
        self.opt.custom_flags(FILE_FLAG_OVERLAPPED | flags);
        self
    }

    pub fn security_qos_flags(&mut self, flags: u32) -> &mut Self {
        self.opt.security_qos_flags(flags);
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        self.opt.open(path).map(|file| File { handle: file })
    }
}

pub struct File {
    handle: StdFile,
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
        self.handle.metadata()
    }

    pub fn set_len(&self, size: u64) -> Result<()> {
        self.handle.set_len(size)
    }

    pub fn set_permissions(&self, perm: Permissions) -> Result<()> {
        self.handle.set_permissions(perm)
    }
}

impl AsHandle for File {
    type Handle = HANDLE;

    fn as_handle(&self) -> Self::Handle {
        self.handle.as_raw_handle() as HANDLE
    }
}

impl Read for File {
    ///
    /// ```
    /// use iocp_rs::{CompletionPort, File, OpenOptions, Read};
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
    ///     let result_list = cmp.get_many(&mut list, None)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn read(&mut self, mut buff: Vec<u8>) -> Result<Context> {
        let len = len(&buff);
        let buff_ptr = buff.as_mut_ptr();
        let mut context = Context::new(self.handle.as_raw_handle() as HANDLE, buff, IOType::Read);
        let over_lapped_ptr = context.over_lapped_ptr();

        let ret = unsafe {
            ReadFile(
                self.handle.as_raw_handle() as HANDLE,
                buff_ptr as *mut _,
                len,
                null_mut(),
                over_lapped_ptr,
            )
        };

        match cvt(ret) {
            Ok(_) => Ok(context),
            Err(e) if e.raw_os_error() == Some(ERROR_IO_PENDING as i32) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let ret = unsafe { CancelIo(self.handle.as_raw_handle() as HANDLE) };

        if ret == 0 {
            panic!("file drop error {:?}", Error::last_os_error());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompletionPort, OpenOptions, Read};

    #[test]
    fn read_file() {
        let cmp = CompletionPort::new(1).unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .open("E:\\rust\\iocp-rs\\test.txt")
            .unwrap();
        cmp.add(1, &file).unwrap();
        let mut buff = vec![0; 10];

        let context = file.read(buff).unwrap();
        let mut list = vec![context];

        let result_list = cmp.get_many(&mut list, None).unwrap();
    }
}
