use crate::fs::File;
use std::fs::OpenOptions as StdOpenOptions;
use std::io::Result;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OVERLAPPED;

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
        self.opt.open(path).map(|file| File { inner: file })
    }
}
