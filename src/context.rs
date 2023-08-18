use std::{
    io::{Result, Error},
    mem::zeroed
};

use windows_sys::Win32::{
    Foundation::{HANDLE, ERROR_NOT_FOUND},
    System::IO::{CancelIoEx, OVERLAPPED},
};

use crate::utils::cvt;

pub enum IOType {
    Read,
    Write,
}

pub struct Context {
    pub(crate) over_lapped: OVERLAPPED,
    pub(crate) buff: Vec<u8>,
    handle: HANDLE,
    pub(crate) io_type: IOType,
}

impl Context {
    pub fn new(handle: HANDLE, buff: Vec<u8>, io_type: IOType) -> Self {
        let over_lapped = unsafe { zeroed::<OVERLAPPED>() };
        Self {
            handle,
            buff,
            io_type,
            over_lapped,
        }
    }

    pub(crate) fn set_offset(&mut self, offset: u64) {
        let low_offset = (offset & (u32::MAX as u64)) as u32;
        let high_offset = (offset >> 32) as u32;

        self.over_lapped.Anonymous.Anonymous.Offset = low_offset;
        self.over_lapped.Anonymous.Anonymous.OffsetHigh = high_offset;
    }

    pub(crate) fn offset(&self) -> u64 {
        let low_offset = unsafe { self.over_lapped.Anonymous.Anonymous.Offset as u64 };
        let high_offset = unsafe { self.over_lapped.Anonymous.Anonymous.OffsetHigh as u64 };

        (high_offset << 32) | low_offset
    }

    pub fn get_buff(&self) -> &[u8] {
        &self.buff
    }

    pub fn io_type(&self) -> &IOType {
        &self.io_type
    }

    pub(crate) fn over_lapped_ptr(&mut self) -> *mut OVERLAPPED {
        (&mut self.over_lapped) as *mut _
    }

    /// 
    /// Cancel this context with handle and overlapped.
    /// There is no guarantee that underlying drivers correctly support cancellation.
    /// ```
    /// use std::net::{TcpStream, TcpListener};
    /// use iocp_rs::{CompletionPort, net::TcpStreamExt, AsHandle};
    /// use std::thread::{spawn, sleep};
    /// use std::os::windows::io::{AsRawSocket, RawSocket};
    /// use std::io::Write;
    /// use std::time::Duration;
    /// use windows_sys::Win32::Foundation::HANDLE;
    /// 
    /// struct MyTcpStream {
    ///     inner: TcpStream
    /// }
    /// 
    /// impl AsRawSocket for MyTcpStream {
    ///     fn as_raw_socket(&self) -> RawSocket {
    ///         self.inner.as_raw_socket()
    ///     }
    /// }
    /// 
    /// impl AsHandle for MyTcpStream {
    ///     type Handle = HANDLE;
    ///     fn as_handle(&self) -> Self::Handle {
    ///         self.inner.as_raw_socket() as HANDLE
    ///     }
    /// }
    /// 
    /// impl TcpStreamExt for MyTcpStream {}
    /// 
    /// fn main() {
    ///     let cmp = CompletionPort::new(1).unwrap();
    ///     let join = spawn(|| {
    ///             let listener = TcpListener::bind("127.0.0.1:999").unwrap();
    ///             let (mut stream, _) = listener.accept().unwrap();
    ///             
    ///             sleep(Duration::from_secs(5));
    ///             let ret = stream.write(b"123");
    ///             dbg!(ret);
    ///     });
    /// 
    ///     let stream = TcpStream::connect("127.0.0.1:999").unwrap();
    ///     let mut stream1 = MyTcpStream {inner: stream};
    ///     cmp.add(1, &stream1).unwrap();
    /// 
    ///     let context = stream1.read(vec![0; 3]).unwrap();
    ///     context.cancel();
    ///     let result = cmp.get(None).unwrap();
    ///     assert_eq!(result.token(), 1);
    /// }
    /// 
    /// ```
    pub fn cancel(self) -> Result<()> {
        let ret = unsafe { CancelIoEx(self.handle, &self.over_lapped as *const _) };

        match cvt(ret) {
            Ok(_) => Ok(()),
            Err(e) if e.raw_os_error() == Some(ERROR_NOT_FOUND as i32) => Ok(()),
            Err(e) => Err(e)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::net::{TcpStream, TcpListener};
    use crate::{CompletionPort, net::TcpStreamExt, AsHandle};
    use std::thread::{spawn, sleep};
    use std::os::windows::io::{AsRawSocket, RawSocket};
    use std::io::Write;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::HANDLE;
    
    struct MyTcpStream {
        inner: TcpStream
    }
    
    impl AsRawSocket for MyTcpStream {
        fn as_raw_socket(&self) -> RawSocket {
            self.inner.as_raw_socket()
        }
    }
    
    impl AsHandle for MyTcpStream {
        fn as_handle(&self) -> HANDLE {
            self.inner.as_raw_socket() as HANDLE
        }
    }
    
    impl TcpStreamExt for MyTcpStream {}
    
    #[test]
    fn cancel() {
        let cmp = CompletionPort::new(1).unwrap();
        let join = spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:999").unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            
            sleep(Duration::from_secs(5));
            let ret = Write::write(&mut stream, b"123");
            // dbg!(ret);
        });
    
        let stream = TcpStream::connect("127.0.0.1:999").unwrap();
        let mut stream1 = MyTcpStream {inner: stream};
        cmp.add(1, &stream1).unwrap();
    
        let context = stream1.read(vec![0; 3]).unwrap();
        context.cancel().unwrap();
        sleep(Duration::from_secs(5));
        let result = cmp.get(None).unwrap();
        assert_eq!(result.token(), 2);
    }
}
