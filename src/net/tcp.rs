use std::io::{Error, Result};
use std::net::{
    Shutdown, SocketAddr, TcpListener as StdTcpListner, TcpStream as StdTcpStream, ToSocketAddrs,
};
use std::os::windows::prelude::AsRawSocket;
use std::time::Duration;

use windows_sys::Win32::Networking::WinSock::{WSARecv, WSASend, WSA_IO_PENDING};
use windows_sys::Win32::{
    Foundation::HANDLE,
    Networking::WinSock::{SOCKET, WSABUF},
};

use crate::context::IOType;
use crate::io::{Read, Write};
use crate::len;
use crate::{AsHandle, Context};

use super::cvt_for_socket;

pub struct TcpListener {
    inner: StdTcpListner,
}

impl TcpListener {
    ///
    /// Creates a new TcpListener which will be bound to the specified address.
    ///
    /// ```
    /// use iocp_rs::net::TcpListener;
    /// use std::io::Result;
    ///
    /// fn main() -> Result<()> {
    ///     TcpListener::bind("127.0.0.1:2098").map(|_| ())
    /// }
    /// ```
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        StdTcpListner::bind(addr).map(|inner| Self { inner })
    }

    pub fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        self.inner
            .accept()
            .map(|(inner, addr)| (TcpStream { inner }, addr))
    }
}

pub struct TcpStream {
    inner: StdTcpStream,
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        StdTcpStream::connect(addr).map(|inner| Self { inner })
    }

    pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> Result<Self> {
        StdTcpStream::connect_timeout(addr, timeout).map(|inner| Self { inner })
    }
}

impl AsHandle for TcpStream {
    type Handle = HANDLE;

    fn as_handle(&self) -> Self::Handle {
        self.inner.as_raw_socket() as HANDLE
    }
}

impl Read for TcpStream {
    fn read(&mut self, mut buff: Vec<u8>) -> Result<Context> {
        let socket = self.inner.as_raw_socket() as SOCKET;
        let buff_len = len(&buff);
        let wsa_buff = WSABUF {
            len: buff_len,
            buf: buff.as_mut_ptr(),
        };
        let handle = self.as_handle();
        let mut context = Context::new(handle, buff, IOType::Read);
        let mut bytes_used = 0;
        let mut flags = 0;

        let ret = unsafe {
            WSARecv(
                socket,
                &wsa_buff,
                1,
                &mut bytes_used,
                &mut flags,
                context.over_lapped_ptr(),
                None,
            )
        };

        match cvt_for_socket(ret) {
            Ok(_) => Ok(context),
            Err(ref e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

impl Write for TcpStream {
    fn write(&self, mut buff: Vec<u8>) -> Result<Context> {
        let socket = self.inner.as_raw_socket() as SOCKET;
        let buff_len = len(&buff);

        let wsa_buff = WSABUF {
            len: buff_len,
            buf: buff.as_mut_ptr(),
        };
        let handle = self.as_handle();
        let mut context = Context::new(handle, buff, IOType::Write);
        let mut bytes_used = 0;

        let ret = unsafe {
            WSASend(
                socket,
                &wsa_buff,
                1,
                &mut bytes_used,
                0,
                context.over_lapped_ptr(),
                None,
            )
        };

        match cvt_for_socket(ret) {
            Ok(_) => Ok(context),
            Err(ref e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok(context),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Write;
    use std::{net::TcpListener, thread::spawn};

    use crate::{fs::OpenOptions, io::Read, CompletionPort};

    use super::TcpStream;

    #[test]
    fn tcp_read() {
        let cmp = CompletionPort::new(2).unwrap();
        let join = spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:999").unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            stream.write(b"hello").unwrap();
        });

        let mut file = OpenOptions::new()
            .read(true)
            .open("..\\test.txt")
            .unwrap();
        let mut stream = TcpStream::connect("127.0.0.1:999").unwrap();

        cmp.add(1, &stream).unwrap();
        cmp.add(2, &file).unwrap();

        let mut map = HashMap::new();
        map.insert(1, stream.read(vec![0; 10]).unwrap());
        map.insert(2, file.read(vec![0; 3]).unwrap());

        loop {
            if map.is_empty() {
                break;
            } else {
                let result_list = cmp.get_many(map.len(), None).unwrap();

                for result in result_list {
                    if let Some(context) = map.remove(&result.token()) {
                        if result.token() == 1 {
                            assert_eq!(&context.get_buff()[..result.bytes_used() as usize], b"hello".as_slice());
                        }
                        if result.token() == 2 {
                            assert_eq!(&context.get_buff()[..result.bytes_used() as usize], b"123".as_slice());
                        }
                    }
                }
            }
        }

        join.join().unwrap();
    }
}
