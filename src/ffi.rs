#![allow(dead_code)]

use std::mem;
use std::net;
use std::ptr;

use libc::{self, c_void, c_char, c_int, c_uint, c_ushort};
use nix::sys::socket;

pub const IFNAMSIZ: usize = 16;

#[repr(C)]
pub struct ifreq_with_hwaddr {
    pub ifr_name: [u8; IFNAMSIZ],
    pub ifr_hwaddr: socket::sockaddr,
}

#[repr(C)]
pub struct ifreq_with_flags {
    pub ifr_name: [u8; IFNAMSIZ],
    pub ifr_flags: c_ushort,
}

#[repr(C)]
pub struct ifreq_with_mtu {
    pub ifr_name: [u8; IFNAMSIZ],
    pub ifr_mtu: c_int,
}

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

fn make_int16(hi: u8, lo: u8) -> u16 {
    (lo as u16) | ((hi as u16) << 8)
}

pub fn convert_sockaddr(sa: *mut socket::sockaddr) -> Option<net::SocketAddr> {
    if sa == ptr::null_mut() {
        return None;
    }

    let (addr, port, flowinfo, scope_id) = match unsafe { *sa }.sa_family as i32 {
        libc::AF_INET => {
            let sa: *const socket::sockaddr_in = unsafe { mem::transmute(sa) };
            let sa = &unsafe { *sa };
            let (addr, port) = (sa.sin_addr.s_addr, sa.sin_port);
            (net::IpAddr::V4(net::Ipv4Addr::new(((addr & 0x000000FF) >> 0) as u8,
                                                ((addr & 0x0000FF00) >> 8) as u8,
                                                ((addr & 0x00FF0000) >> 16) as u8,
                                                ((addr & 0xFF000000) >> 24) as u8)),
             port, 0, 0)
        }
        libc::AF_INET6 => {
            let sa: *const socket::sockaddr_in6 = unsafe { mem::transmute(sa) };
            let sa = &unsafe { *sa };
            let (addr, port, flowinfo, scope_id) = (sa.sin6_addr.s6_addr, sa.sin6_port, sa.sin6_flowinfo, sa.sin6_scope_id);
            (net::IpAddr::V6(net::Ipv6Addr::new(make_int16(addr[0], addr[1]),
                                                make_int16(addr[2], addr[3]),
                                                make_int16(addr[4], addr[5]),
                                                make_int16(addr[6], addr[7]),
                                                make_int16(addr[8], addr[9]),
                                                make_int16(addr[10], addr[11]),
                                                make_int16(addr[12], addr[13]),
                                                make_int16(addr[14], addr[15]),
                                                )),
             port, flowinfo, scope_id)
        }
        _ => return None,
    };

    let sa = match addr {
        net::IpAddr::V4(addr) => net::SocketAddr::V4(net::SocketAddrV4::new(addr, port)),
        net::IpAddr::V6(addr) => net::SocketAddr::V6(net::SocketAddrV6::new(addr, port, flowinfo, scope_id)),
    };
    Some(sa)
}

// Helper functions from `helpers.c`
extern "C" {
    #[cfg(target_os = "macos")]
    pub fn rust_LLADDR(p: *mut ifaddrs) -> *const u8;
}

#[cfg(test)]
mod tests {
    use super::make_int16;

    #[test]
    fn test_make_int16() {
        assert_eq!(make_int16(0xff, 0x00), 0xff00);
    }
}
