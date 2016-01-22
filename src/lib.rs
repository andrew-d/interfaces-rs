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

pub use error::InterfacesError;
pub use flags::InterfaceFlags;

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
    Ipv4,
    Ipv6,
    Link,
    Unknown(i32),

    // Linux only
    Packet,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Ipv4 => write!(f, "IPv4"),
            Kind::Ipv6 => write!(f, "IPv6"),
            Kind::Link => write!(f, "Link"),
            Kind::Unknown(v) => write!(f, "Unknown({})", v),
            Kind::Packet => write!(f, "Packet"),
        }
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

    /// The underlying socket address, if it applies.
    pub addr: Option<net::SocketAddr>,

    /// The netmask of for this interface address, if it applies.
    pub mask: Option<net::SocketAddr>,

    /// The broadcast address or destination address, if it applies.
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

    /// Interface flags.
    ///
    /// NOTE: The underlying API returns this value for each address of an interface, not each
    /// interface itself.  We assume that they are all equal and take the first set of flags (from
    /// the first address).
    pub flags: InterfaceFlags,
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
                // Either get the current entry for this interface or insert a new one.
                let iface = ifs
                    .entry(name)
                    .or_insert_with(|| convert_ifaddr(cur));
                
                // If we can, convert this current address.
                if let Some(addr) = convert_ifaddr_address(cur) {
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

    /// Returns whether this interface is up.
    pub fn is_up(&self) -> bool {
        self.flags.contains(flags::IFF_UP)
    }
}

fn convert_ifaddr_name(ifa: *mut ffi::ifaddrs) -> Option<String> {
    let ifa = unsafe { &mut *ifa };
    match unsafe { CStr::from_ptr(ifa.ifa_name).to_str() } {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
    }
}

fn convert_ifaddr(ifa: *mut ffi::ifaddrs) -> Interface {
    let ifa = unsafe { &mut *ifa };

    // NOTE: can unwrap() here since we only call this function if the prior call to
    // convert_ifaddr_name succeeded.  It's a bit sad that we have to duplicate work, but not a
    // huge deal.
    let name = convert_ifaddr_name(ifa).unwrap();

    let flags = InterfaceFlags::from_bits_truncate(ifa.ifa_flags);

    Interface {
        name: name,
        addresses: vec![],
        flags: flags,
    }
}

// This is a bit scary, but the various address families are different from platform to platform,
// and also from OS version to OS version.  Essentially, we have a couple of families that we know
// about (IPv4, IPv6, etc.), and a couple that we determined at build time by compiling some C code
// that tried to include the value of the AF_* constant.  For each of these, we try getting the
// corresponding constant, and then verify if it matches.
fn convert_ifaddr_family(family: i32) -> Kind {
    // Helper macro!
    macro_rules! check_family {
        ($cc:tt -> $ty:ident) => {
            if let Some(val) = constants::get_constant(stringify!($cc)) {
                if family == val as i32 {
                    return Kind::$ty;
                }
            }
        };
    }

    check_family!(AF_PACKET -> Packet);
    check_family!(AF_LINK -> Link);

    match family {
        socket::AF_INET => Kind::Ipv4,
        socket::AF_INET6 => Kind::Ipv6,
        val => Kind::Unknown(val),
    }
}

fn convert_ifaddr_address(ifa: *mut ffi::ifaddrs) -> Option<Address> {
    let ifa = unsafe { &mut *ifa };

    let kind = if ifa.ifa_addr != ptr::null_mut() {
        let fam = unsafe { *ifa.ifa_addr }.sa_family as i32;
        convert_ifaddr_family(fam)
    } else {
        return None;
    };

    let addr = ffi::convert_sockaddr(ifa.ifa_addr);

    let mask = ffi::convert_sockaddr(ifa.ifa_netmask);

    let flags = InterfaceFlags::from_bits_truncate(ifa.ifa_flags);
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

impl PartialEq for Interface {
    fn eq(&self, other: &Interface) -> bool {
        self.name == other.name
    }
}

impl Eq for Interface {}
