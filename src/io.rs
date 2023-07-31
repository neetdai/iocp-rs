use std::io::Result;

use crate::Context;

pub trait Read {
    fn read(&mut self, buff: Vec<u8>) -> Result<Context>;
}

pub trait ReadAt {
    fn read_at(&mut self, buff: Vec<u8>, offset: u64) -> Result<Context>;
}

pub trait Write {
    fn write(&self, buff: Vec<u8>) -> Result<Context>;
}

pub trait WriteAt {
    fn write_at(&self, buff: Vec<u8>, offset: u64) -> Result<Context>;
}
