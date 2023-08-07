use std::io::ErrorKind;
use std::io::{Error, Result};
use std::mem::zeroed;
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::windows::prelude::AsRawSocket;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Networking::WinSock::WSASend;
use windows_sys::Win32::Networking::WinSock::WSASendTo;
use windows_sys::Win32::Networking::WinSock::{
    WSARecv, WSARecvFrom, SOCKADDR, SOCKET, WSABUF, WSA_IO_PENDING,
};

use crate::context::IOType;
use crate::utils::len;
use crate::{AsHandle, Context};

use super::cvt_for_socket;
use super::SocketAddrCRepr;

pub trait UdpSocketExt: AsRawSocket + AsHandle<Handle = HANDLE> {
    fn recv(&self, mut buff: Vec<u8>) -> Result<Context> {
        let mut wsa_buf = WSABUF {
            len: len(&buff),
            buf: buff.as_mut_ptr(),
        };
        let mut flags = 0;
        let mut bytes_used = 0;
        let handle = self.as_handle();
        let mut context = Context::new(handle, buff, IOType::Read);
        let over_lapped_ptr = context.over_lapped_ptr();

        let ret = unsafe {
            WSARecv(
                self.as_raw_socket() as SOCKET,
                &mut wsa_buf,
                1,
                &mut bytes_used,
                &mut flags,
                over_lapped_ptr,
                None,
            )
        };

        match cvt_for_socket(ret) {
            Ok(_) => Ok(context),
            Err(ref e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok(context),
            Err(e) => Err(e),
        }
    }

    fn recv_from(&self, mut buff: Vec<u8>) -> Result<(Context, SocketAddr)> {
        let wsa_buf = WSABUF {
            len: len(&buff),
            buf: buff.as_mut_ptr(),
        };
        let mut byte_used = 0;
        let mut flag = 0;
        let handle = self.as_handle();
        let mut context = Context::new(handle, buff, IOType::Read);
        let over_lapped_ptr = context.over_lapped_ptr();
        let mut socket_addr = unsafe { zeroed::<SOCKADDR>() };
        let mut socket_addr_len = 0;

        let ret = unsafe {
            WSARecvFrom(
                self.as_raw_socket() as SOCKET,
                &wsa_buf,
                1,
                &mut byte_used,
                &mut flag,
                &mut socket_addr,
                &mut socket_addr_len,
                over_lapped_ptr,
                None,
            )
        };

        let socket_addr =
            unsafe { SocketAddrCRepr::ptrs_to_socket_addr(&socket_addr, socket_addr_len) }.unwrap();

        match cvt_for_socket(ret) {
            Ok(_) => Ok((context, socket_addr)),
            Err(e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok((context, socket_addr)),
            Err(e) => Err(e),
        }
    }

    fn send(&self, mut buff: Vec<u8>) -> Result<Context> {
        let wsa_buf = WSABUF {
            len: len(&buff),
            buf: buff.as_mut_ptr(),
        };
        let mut bytes_used = 0;
        let mut context = Context::new(self.as_handle(), buff, IOType::Write);

        let ret = unsafe {
            WSASend(
                self.as_raw_socket() as SOCKET,
                &wsa_buf,
                1,
                &mut bytes_used,
                0,
                context.over_lapped_ptr(),
                None,
            )
        };

        match cvt_for_socket(ret) {
            Ok(_) => Ok(context),
            Err(e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok(context),
            Err(e) => Err(e),
        }
    }

    fn send_to<A: ToSocketAddrs>(&self, mut buff: Vec<u8>, addr: A) -> Result<Context> {
        let wsa_buf = WSABUF {
            len: len(&buff),
            buf: buff.as_mut_ptr(),
        };
        let mut bytes_used = 0;
        let mut context = Context::new(self.as_handle(), buff, IOType::Write);
        let socket_addr = addr.to_socket_addrs()?.next().ok_or(Error::new(
            ErrorKind::InvalidInput,
            "no addresses to send data to",
        ))?;
        let (socket_addr_ptr, ptr_len) = SocketAddrCRepr::socket_addr_to_ptrs(&socket_addr);

        let ret = unsafe {
            WSASendTo(
                self.as_raw_socket() as SOCKET,
                &wsa_buf,
                1,
                &mut bytes_used,
                0,
                &socket_addr_ptr as *const _ as *const _,
                ptr_len,
                context.over_lapped_ptr(),
                None,
            )
        };

        match cvt_for_socket(ret) {
            Ok(_) => Ok(context),
            Err(e) if e.raw_os_error() == Some(WSA_IO_PENDING) => Ok(context),
            Err(e) => Err(e),
        }
    }

}
