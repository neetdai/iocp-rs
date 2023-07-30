use std::io::Result;

use crate::Context;

pub trait Read {
    fn read(&mut self, buff: Vec<u8>) -> Result<Context>;
}
