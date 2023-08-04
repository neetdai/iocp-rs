mod socket;
mod tcp;
// mod udp;

use std::io::{Error, Result};
use std::mem::zeroed;
use std::mem::{size_of, size_of_val};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::OnceLock;
use windows_sys::Win32::Networking::WinSock::{
    WSACleanup, WSAGetLastError, WSAStartup, AF_INET, AF_INET6, IN6_ADDR, IN6_ADDR_0, IN_ADDR,
    IN_ADDR_0, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6, SOCKADDR_IN6_0, SOCKET_ERROR, WSADATA,
};

pub(crate) struct WSAInit;

impl WSAInit {
    pub(crate) fn new() -> Self {
        unsafe {
            let mut data = zeroed::<WSADATA>();
            let inner = WSAStartup(0x202, &mut data);

            if inner != 0 {
                let code = WSAGetLastError();
                panic!("wsa init error {}", Error::from_raw_os_error(code));
            }
            Self {}
        }
    }
}

impl Drop for WSAInit {
    fn drop(&mut self) {
        unsafe {
            let ret = WSACleanup();
            if ret != 0 {
                let code = WSAGetLastError();
                panic!("wsa drop error {}", Error::from_raw_os_error(code))
            }
        }
    }
}

pub(crate) static ONCE_INIT: OnceLock<WSAInit> = OnceLock::new();

pub(crate) union SocketAddrCRepr {
    v4: SOCKADDR_IN,
    v6: SOCKADDR_IN6,
}

impl SocketAddrCRepr {
    pub(crate) fn socket_addr_to_ptrs(addr: &SocketAddr) -> (Self, i32) {
        match *addr {
            SocketAddr::V4(ref v4) => {
                let sockaddr_in = SOCKADDR_IN {
                    sin_family: AF_INET,
                    sin_port: v4.port(),
                    sin_addr: IN_ADDR {
                        S_un: IN_ADDR_0 {
                            S_addr: u32::from_ne_bytes(v4.ip().octets()),
                        },
                    },
                    sin_zero: [0; 8],
                };

                let sockaddr_in_size = size_of_val(&sockaddr_in) as i32;

                (Self { v4: sockaddr_in }, sockaddr_in_size)
            }
            SocketAddr::V6(ref v6) => {
                let sockaddr_in = SOCKADDR_IN6 {
                    sin6_family: AF_INET6,
                    sin6_port: v6.port(),
                    sin6_addr: IN6_ADDR {
                        u: IN6_ADDR_0 {
                            Byte: v6.ip().octets(),
                        },
                    },
                    sin6_flowinfo: v6.flowinfo(),
                    Anonymous: SOCKADDR_IN6_0 {
                        sin6_scope_id: v6.scope_id(),
                    },
                };

                let sockaddr_in_size = size_of_val(&sockaddr_in) as i32;

                (Self { v6: sockaddr_in }, sockaddr_in_size)
            }
        }
    }

    pub(crate) fn as_ptr(&self) -> *const SOCKADDR {
        self as *const _ as *const _
    }

    pub(crate) unsafe fn ptrs_to_socket_addr(ptr: *const SOCKADDR, len: i32) -> Option<SocketAddr> {
        if (len as usize) < size_of::<i32>() {
            return None;
        }
        match (*ptr).sa_family as _ {
            AF_INET if len as usize >= size_of::<SOCKADDR_IN>() => {
                let b = &*(ptr as *const SOCKADDR_IN);
                let ip = b.sin_addr.S_un.S_addr.to_be();
                let ip = Ipv4Addr::new(
                    (ip >> 24) as u8,
                    (ip >> 16) as u8,
                    (ip >> 8) as u8,
                    ip as u8,
                );
                Some(SocketAddr::V4(SocketAddrV4::new(ip, b.sin_port.to_be())))
            }
            AF_INET6 if len as usize >= size_of::<SOCKADDR_IN6>() => {
                let b = &*(ptr as *const SOCKADDR_IN6);
                let arr = &b.sin6_addr.u.Byte;
                let ip = Ipv6Addr::new(
                    ((arr[0] as u16) << 8) | (arr[1] as u16),
                    ((arr[2] as u16) << 8) | (arr[3] as u16),
                    ((arr[4] as u16) << 8) | (arr[5] as u16),
                    ((arr[6] as u16) << 8) | (arr[7] as u16),
                    ((arr[8] as u16) << 8) | (arr[9] as u16),
                    ((arr[10] as u16) << 8) | (arr[11] as u16),
                    ((arr[12] as u16) << 8) | (arr[13] as u16),
                    ((arr[14] as u16) << 8) | (arr[15] as u16),
                );
                let addr = SocketAddrV6::new(
                    ip,
                    b.sin6_port.to_be(),
                    b.sin6_flowinfo.to_be(),
                    b.Anonymous.sin6_scope_id.to_be(),
                );
                Some(SocketAddr::V6(addr))
            }
            _ => None,
        }
    }
}

pub(crate) fn cvt_for_socket(ret: i32) -> Result<i32> {
    if ret == SOCKET_ERROR {
        let code = unsafe { WSAGetLastError() };
        Err(Error::from_raw_os_error(code))
    } else {
        Ok(ret)
    }
}
