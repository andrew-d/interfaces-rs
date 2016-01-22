#[macro_use] extern crate bitflags;
extern crate ip;
#[macro_use] extern crate lazy_static;
extern crate libc;
extern crate nix;

use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::net;
use std::ptr;

use nix::sys::socket;

use error::InterfacesError;

mod constants;
mod error;
mod ffi;
mod flags;

/// A specialized Result type for this crate.
pub type Result<T> = ::std::result::Result<T, InterfacesError>;

/// `Kind` represents the interface family (equivalent to the `sa_family` field in the `sockaddr`
/// structure).
#[derive(PartialEq, Eq, Debug)]
pub enum Kind {
    Packet,
    Ipv4,
    Ipv6,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Kind::Packet => "Packet",
            Kind::Ipv4 => "IPv4",
            Kind::Ipv6 => "IPv6",
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum NextHop {
    Broadcast(net::SocketAddr),
    Destination(net::SocketAddr),
}

impl fmt::Display for NextHop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NextHop::Broadcast(ref addr) => {
                write!(f, "Broadcast({})", addr)
            },
            NextHop::Destination(ref addr) => {
                write!(f, "Destination({})", addr)
            },
        }
    }
}

/// This structure represents a single address for a given interface.
#[derive(Debug)]
pub struct Address {
    /// The kind of address this is.
    pub kind: Kind,

    /// The address of this interface, if it has one.
    pub addr: Option<net::SocketAddr>,

    /// The netmask of this interface, if it has one.
    pub mask: Option<net::SocketAddr>,

    /// The broadcast address or destination address, if it has one.
    pub hop: Option<NextHop>,
}

/// The `Interface` structure represents a single interface on the system.  It also contains
/// methods to control the interface.
#[derive(Debug)]
pub struct Interface {
    /// The name of this interface.
    pub name: String,

    /// All addresses for this interface.
    pub addresses: Vec<Address>,
}

impl Interface {
    /// Retrieve a list of all interfaces on this system.
    pub fn get_all() -> Result<Vec<Interface>> {
        // Get all interface addresses
        let mut ifap: *mut ffi::ifaddrs = unsafe { mem::zeroed() };
        if unsafe { ffi::getifaddrs(&mut ifap as *mut _) } != 0 {
            return Err(InterfacesError::last_errno());
        }

        // Used to deduplicate interfaces.
        let mut ifs = HashMap::new();

        // Map each interface address to a single interface name.
        let mut cur: *mut ffi::ifaddrs = ifap;
        while cur != ptr::null_mut() {
            if let Some(name) = convert_ifaddr_name(cur) {
                // TODO: this is pretty ugly - there has got to be a nicer way to either use the
                // existing key, or insert it again.
                let nc = name.clone();

                // Either get the current entry for this interface or insert a new one.
                let iface = ifs
                    .entry(name)
                    .or_insert_with(|| Interface {
                        name: nc,
                        addresses: vec![],
                    });
                
                // If we can, convert this current address.
                if let Some(addr) = convert_ifaddr(cur) {
                    iface.addresses.push(addr);
                }
            }

            // TODO: do something else maybe?
            cur = unsafe { (*cur).ifa_next };
        }

        unsafe { ffi::freeifaddrs(ifap) };

        let ret = ifs.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        Ok(ret)
    }
}

fn convert_ifaddr_name(ifa: *mut ffi::ifaddrs) -> Option<String> {
    let ifa = unsafe { &mut *ifa };
    match unsafe { CStr::from_ptr(ifa.ifa_name).to_str() } {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
    }
}

fn convert_ifaddr(ifa: *mut ffi::ifaddrs) -> Option<Address> {
    let ifa = unsafe { &mut *ifa };

    let kind = if ifa.ifa_addr != ptr::null_mut() {
        match unsafe { *ifa.ifa_addr }.sa_family as i32 {
            ffi::AF_PACKET => Kind::Packet,
            socket::AF_INET => Kind::Ipv4,
            socket::AF_INET6 => Kind::Ipv6,
            _ => return None,
        }
    } else {
        return None;
    };

    let addr = ffi::convert_sockaddr(ifa.ifa_addr);

    let mask = ffi::convert_sockaddr(ifa.ifa_netmask);

    let flags = flags::InterfaceFlags::from_bits_truncate(ifa.ifa_flags);
    let hop = if flags.contains(flags::IFF_BROADCAST) {
        match ffi::convert_sockaddr(ifa.ifa_ifu.ifu_broadaddr()) {
            Some(x) => Some(NextHop::Broadcast(x)),
            None => None,
        }
    } else {
        match ffi::convert_sockaddr(ifa.ifa_ifu.ifu_dstaddr()) {
            Some(x) => Some(NextHop::Destination(x)),
            None => None,
        }
    };

    Some(Address {
        kind: kind,
        addr: addr,
        mask: mask,
        hop: hop,
    })
}
