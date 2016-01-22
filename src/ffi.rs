use std::mem;
use std::net;
use std::ptr;

use ip::IpAddr;
use libc::{c_void, c_char, c_int, c_uint};
use nix::sys::socket;

// nix doesn't have this const
pub const AF_PACKET: i32 = 17;

#[repr(C)]
pub struct union_ifa_ifu {
    pub data: *mut c_void,
}
impl union_ifa_ifu {
    pub fn ifu_broadaddr(&mut self) -> *mut socket::sockaddr {
        self.data as *mut socket::sockaddr
    }
    pub fn ifu_dstaddr(&mut self) -> *mut socket::sockaddr {
        self.data as *mut socket::sockaddr
    }
}

#[repr(C)]
pub struct ifaddrs {
    pub ifa_next: *mut ifaddrs,
    pub ifa_name: *mut c_char,
    pub ifa_flags: c_uint,
    pub ifa_addr: *mut socket::sockaddr,
    pub ifa_netmask: *mut socket::sockaddr,
    pub ifa_ifu: union_ifa_ifu,
    pub ifa_data: *mut c_void,
}

extern "C" {
    pub fn getifaddrs(ifap: *mut *mut ifaddrs) -> c_int;
    pub fn freeifaddrs(ifa: *mut ifaddrs) -> c_void;
}

pub fn convert_sockaddr(sa: *mut socket::sockaddr) -> Option<net::SocketAddr> {
    if sa == ptr::null_mut() {
        return None;
    }

    let (addr, port) = match unsafe { *sa }.sa_family as i32 {
        socket::AF_INET => {
            let sa: *const socket::sockaddr_in = unsafe { mem::transmute(sa) };
            let sa = &unsafe { *sa };
            let (addr, port) = (sa.sin_addr.s_addr, sa.sin_port);
            (IpAddr::V4(net::Ipv4Addr::new(((addr & 0x000000FF) >> 0) as u8,
                                           ((addr & 0x0000FF00) >> 8) as u8,
                                           ((addr & 0x00FF0000) >> 16) as u8,
                                           ((addr & 0xFF000000) >> 24) as u8)),
             port)
        }
        socket::AF_INET6 => {
            let sa: *const socket::sockaddr_in6 = unsafe { mem::transmute(sa) };
            let sa = &unsafe { *sa };
            let (addr, port) = (sa.sin6_addr.s6_addr, sa.sin6_port);
            (IpAddr::V6(net::Ipv6Addr::new(addr[0],
                                           addr[1],
                                           addr[2],
                                           addr[3],
                                           addr[4],
                                           addr[5],
                                           addr[6],
                                           addr[7])),
             port)
        }
        _ => return None,
    };

    let sa = match addr {
        IpAddr::V4(addr) => net::SocketAddr::V4(net::SocketAddrV4::new(addr, port)),
        IpAddr::V6(addr) => net::SocketAddr::V6(net::SocketAddrV6::new(addr, port, 0, 0)),
    };
    Some(sa)
}
