#![deny(missing_docs)]

//! A library for interoperating with the network interfaces of a system.
//!
//! TODO: add more documentation on how to use.

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate nix;

use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::net;
use std::ptr;

use libc::c_int;
use libc::{AF_INET, SOCK_DGRAM};
use libc::{close, ioctl, socket};

#[cfg(target_os = "linux")]
use nix::sys::socket;

pub use error::InterfacesError;
pub use flags::InterfaceFlags;

mod constants;
mod error;
mod ffi;

/// Submodule containing various flags.
pub mod flags;

/// A specialized Result type for this crate.
pub type Result<T> = ::std::result::Result<T, InterfacesError>;

/// `Kind` represents the interface family (equivalent to the `sa_family` field in the `sockaddr`
/// structure).
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Kind {
    /// This interface is IPv4.
    Ipv4,

    /// This interface is IPv6.
    Ipv6,

    /// This interface is a link interface (`AF_LINK`).
    Link,

    /// This interface has an unknown interface type.  The interior `i32` contains the numerical
    /// value that is unknown.
    Unknown(i32),

    /// Linux only: this interface is a packet interface (`AF_PACKET`).
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

/// The next hop for an interface.  See the individual variants for more information.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NextHop {
    /// The broadcast address associated with the interface's address.
    Broadcast(net::SocketAddr),

    /// The destination address of a point-to-point interface.
    Destination(net::SocketAddr),
}

impl fmt::Display for NextHop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NextHop::Broadcast(ref addr) => write!(f, "Broadcast({})", addr),
            NextHop::Destination(ref addr) => write!(f, "Destination({})", addr),
        }
    }
}

/// This structure represents a single address for a given interface.
#[derive(Debug, Clone, Copy)]
pub struct Address {
    /// The kind of address this is (e.g. IPv4).
    pub kind: Kind,

    /// The underlying socket address, if it applies.
    pub addr: Option<net::SocketAddr>,

    /// The netmask of this interface address, if it applies.
    pub mask: Option<net::SocketAddr>,

    /// The broadcast address or destination address, if it applies.
    pub hop: Option<NextHop>,
}

/// HardwareAddr represents a hardware address (commonly known as a MAC address) of a given
/// interface.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct HardwareAddr([u8; 6]);

impl HardwareAddr {
    /// Returns a new, empty `HardwareAddr` structure.  This is equivalent to the MAC address
    /// `00:00:00:00:00:00`.
    pub fn zero() -> HardwareAddr {
        HardwareAddr([0; 6])
    }

    /// Formats this hardware address in the standard MAC address format - 6 octets in hexadecimal
    /// format, each seperated by a colon.
    ///
    /// ```
    /// # use interfaces2::HardwareAddr;
    /// let s = HardwareAddr::zero().as_string();
    /// assert_eq!(s, "00:00:00:00:00:00");
    /// ```
    pub fn as_string(&self) -> String {
        let &HardwareAddr(ref arr) = self;

        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            arr[0], arr[1], arr[2], arr[3], arr[4], arr[5],
        )
    }

    /// Formats this hardware address as a sequence of hexadecimal numbers without the seperating
    /// colons.
    ///
    /// ```
    /// # use interfaces2::HardwareAddr;
    /// let s = HardwareAddr::zero().as_bare_string();
    /// assert_eq!(s, "000000000000");
    /// ```
    pub fn as_bare_string(&self) -> String {
        let &HardwareAddr(ref arr) = self;

        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            arr[0], arr[1], arr[2], arr[3], arr[4], arr[5],
        )
    }

    /// Returns the raw bytes representing this hardware address.
    ///
    /// ```
    /// # use interfaces2::HardwareAddr;
    /// let s = HardwareAddr::zero();
    /// assert_eq!(s.as_bytes(), &[0, 0, 0, 0, 0, 0]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        let &HardwareAddr(ref arr) = self;
        arr
    }
}

impl fmt::Display for HardwareAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// An iterator to walk through all `ifaddrs`.
struct IfAddrIterator {
    orig: *mut ffi::ifaddrs,
    ifap: *mut ffi::ifaddrs,
}

impl IfAddrIterator {
    fn new() -> Result<IfAddrIterator> {
        // Get all interface addresses
        let mut ifap: *mut ffi::ifaddrs = unsafe { mem::zeroed() };
        if unsafe { ffi::getifaddrs(&mut ifap as *mut _) } != 0 {
            return Err(InterfacesError::last_os_error());
        }

        Ok(IfAddrIterator {
            orig: ifap,
            ifap: ifap,
        })
    }
}

impl Iterator for IfAddrIterator {
    type Item = *mut ffi::ifaddrs;

    fn next(&mut self) -> Option<*mut ffi::ifaddrs> {
        if self.ifap.is_null() {
            return None;
        }

        let ret = self.ifap;
        self.ifap = unsafe { (*self.ifap).ifa_next };
        Some(ret)
    }
}

impl Drop for IfAddrIterator {
    fn drop(&mut self) {
        // Zero the iterator pointer.
        self.ifap = ptr::null_mut();

        // Zero the original pointer in the structure and then free it.
        let ptr = mem::replace(&mut self.orig, ptr::null_mut());
        unsafe { ffi::freeifaddrs(ptr) };
    }
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

    // Information socket
    sock: c_int,
}

impl Interface {
    /// Retrieve a list of all interfaces on this system.
    pub fn get_all() -> Result<Vec<Interface>> {
        // Map each interface address to a single interface name.
        let mut ifs = HashMap::new();
        for cur in try!(IfAddrIterator::new()) {
            // Only support interfaces with valid names.
            let ifname = match convert_ifaddr_name(cur) {
                Some(n) => n,
                None => continue,
            };

            let iface = if ifs.contains_key(&ifname) {
                ifs.get_mut(&ifname).unwrap()
            } else {
                let new_if = match Interface::new_from_ptr(cur) {
                    Ok(i) => i,
                    Err(_) => continue,
                };
                ifs.insert(ifname.clone(), new_if);
                ifs.get_mut(&ifname).unwrap()
            };

            // If we can, convert this current address.
            if let Some(addr) = convert_ifaddr_address(cur) {
                iface.addresses.push(addr);
            }
        }

        let ret = ifs.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        Ok(ret)
    }

    /// Returns an `Interface` instance representing the interface with the given name.  Will
    /// return `Ok(Some(Interface))` on success, `Ok(None)` if there is no such interface, and
    /// `Err(..)` on failure.
    ///
    /// ```
    /// # use interfaces2::{Interface, Result};
    /// # fn foo() -> Result<Option<Interface>> {
    /// let iface = try!(Interface::get_by_name("lo"));
    /// if let Some(ref lo) = iface {
    ///     assert!(lo.is_loopback());
    /// } else {
    ///     println!("Could not find loopback interface");
    /// }
    /// # Ok(iface)
    /// # }
    /// ```
    pub fn get_by_name(name: &str) -> Result<Option<Interface>> {
        let mut ret = None;

        for cur in try!(IfAddrIterator::new()) {
            // Only support interfaces with valid names.
            let ifname = match convert_ifaddr_name(cur) {
                Some(n) => n,
                None => continue,
            };

            if ifname != name {
                continue;
            }

            // Get or create the Interface
            let mut i = match ret.take() {
                Some(i) => i,
                None => try!(Interface::new_from_ptr(cur)),
            };

            // If we can, convert this current address.
            if let Some(addr) = convert_ifaddr_address(cur) {
                i.addresses.push(addr);
            }

            ret = Some(i);
        }

        Ok(ret)
    }

    /// Create a new Interface from a given `ffi::ifaddrs`.
    fn new_from_ptr(ifa: *mut ffi::ifaddrs) -> Result<Interface> {
        let ifa = unsafe { &mut *ifa };

        // NOTE: can unwrap() here since we only call this function if the prior call to
        // convert_ifaddr_name succeeded.  It's a bit sad that we have to duplicate work, but not a
        // huge deal.
        let name = convert_ifaddr_name(ifa).unwrap();

        // Try to create a socket that we use to get info about this interface.
        let sock = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if sock < 0 {
            return Err(InterfacesError::last_os_error());
        }

        let flags = InterfaceFlags::from_bits_truncate(ifa.ifa_flags);
        Ok(Interface {
            name: name,
            addresses: vec![],
            flags: flags,
            sock: sock,
        })
    }

    /// Returns whether this interface is up.
    pub fn is_up(&self) -> bool {
        self.flags.contains(InterfaceFlags::IFF_UP)
    }

    /// Returns whether this interface is a loopback address.
    pub fn is_loopback(&self) -> bool {
        self.flags.contains(InterfaceFlags::IFF_LOOPBACK)
    }

    /// Retrieves the hardware address of this interface.
    pub fn hardware_addr(&self) -> Result<HardwareAddr> {
        self.hardware_addr_impl()
    }

    #[cfg(target_os = "linux")]
    #[allow(non_snake_case)]
    fn hardware_addr_impl(&self) -> Result<HardwareAddr> {
        // We need this IOCTL in order to get the hardware address.
        let SIOCGIFHWADDR = match constants::get_constant("SIOCGIFHWADDR") {
            Some(c) => c,
            None => return Err(InterfacesError::NotSupported("SIOCGIFHWADDR")),
        };

        let mut req = ffi::ifreq_with_hwaddr {
            ifr_name: [0; ffi::IFNAMSIZ],
            ifr_hwaddr: socket::sockaddr {
                sa_family: 0,
                sa_data: [0; 14],
            },
        };

        copy_slice(&mut req.ifr_name, self.name.as_bytes());

        let res = unsafe { ioctl(self.sock, SIOCGIFHWADDR, &mut req) };
        if res < 0 {
            return Err(InterfacesError::last_os_error());
        }

        let mut addr = [0; 6];
        for i in 0..6 {
            addr[i] = req.ifr_hwaddr.sa_data[i];
        }

        // Hardware addresses are `i8`s on some Linux versions for some reason?
        let addr = unsafe { mem::transmute::<[_; 6], [u8; 6]>(addr) };
        Ok(HardwareAddr(addr))
    }

    #[cfg(target_os = "macos")]
    #[allow(non_snake_case)]
    fn hardware_addr_impl(&self) -> Result<HardwareAddr> {
        // We need certain constants - get them now.
        let AF_LINK = match constants::get_constant("AF_LINK") {
            Some(c) => c as i32,
            None => return Err(InterfacesError::NotSupported("AF_LINK")),
        };

        // Walk all interfaces looking for one that is the right type and name.  We:
        //  - Get the name from this interface
        //  - Filter only where the name == ours
        //  - Get only AF_LINK interfaces
        let mut it = try!(IfAddrIterator::new())
            .filter_map(|cur| {
                if let Some(name) = convert_ifaddr_name(cur) {
                    Some((name, cur))
                } else {
                    None
                }
            })
            .filter(|&(ref name, _)| name == &self.name)
            .filter(|&(_, ifa)| {
                let ifa = unsafe { &mut *ifa };
                let family = unsafe { *ifa.ifa_addr }.sa_family as i32;
                family == AF_LINK
            });

        let link_if = match it.next() {
            Some((_, ifa)) => ifa,
            None => return Err(InterfacesError::NotSupported("No AF_LINK")),
        };

        let mut addr = [0; 6];
        let mut pr = unsafe { ffi::rust_LLADDR(link_if) };

        for i in 0..6 {
            addr[i] = unsafe { *pr };
            pr = ((pr as usize) + 1) as *const u8;
        }

        drop(it);
        Ok(HardwareAddr(addr))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn hardware_addr_impl(&self) -> Result<HardwareAddr> {
        Err(InterfacesError::NotSupported("Unknown OS"))
    }

    /// Sets the interface as up or down.  This will change the status of the given interface in
    /// the system, and update the flags of this `Interface` instance.
    #[allow(non_snake_case)]
    pub fn set_up(&mut self, up: bool) -> Result<()> {
        // We need these IOCTLs in order to get/set the interface flags.
        let SIOCGIFFLAGS = match constants::get_constant("SIOCGIFFLAGS") {
            Some(c) => c,
            None => return Err(InterfacesError::NotSupported("SIOCGIFFLAGS")),
        };
        let SIOCSIFFLAGS = match constants::get_constant("SIOCSIFFLAGS") {
            Some(c) => c,
            None => return Err(InterfacesError::NotSupported("SIOCSIFFLAGS")),
        };

        let mut req = ffi::ifreq_with_flags {
            ifr_name: [0; ffi::IFNAMSIZ],
            ifr_flags: 0,
        };

        copy_slice(&mut req.ifr_name, self.name.as_bytes());

        // Get the existing flags.
        let res = unsafe { ioctl(self.sock, SIOCGIFFLAGS, &mut req) };
        if res < 0 {
            let err = InterfacesError::last_os_error();
            return Err(err);
        }

        // Depending on our up/down, clear IFF_UP.
        // NOTE: we don't want to convert this to/from an InterfaceFlags variable, since that will
        // strip out any unknown bits (which we don't want).  So, we just use good old bitwise
        // operators to set/clear the flags.
        let flag_val = InterfaceFlags::IFF_UP.bits() as u16;
        req.ifr_flags = if up {
            req.ifr_flags | flag_val
        } else {
            req.ifr_flags & (!flag_val)
        };

        // Set the flags back.
        let res = unsafe { ioctl(self.sock, SIOCSIFFLAGS, &mut req) };
        if res < 0 {
            return Err(InterfacesError::last_os_error());
        }

        // Update our flags to represent the new state.
        self.flags = InterfaceFlags::from_bits_truncate(req.ifr_flags as u32);

        Ok(())
    }

    /// Retrieve the MTU of this interface.
    #[allow(non_snake_case)]
    pub fn get_mtu(&self) -> Result<u32> {
        let SIOCGIFMTU = match constants::get_constant("SIOCGIFMTU") {
            Some(c) => c,
            None => return Err(InterfacesError::NotSupported("SIOCGIFMTU")),
        };

        let mut req = ffi::ifreq_with_mtu {
            ifr_name: [0; ffi::IFNAMSIZ],
            ifr_mtu: 0,
        };

        copy_slice(&mut req.ifr_name, self.name.as_bytes());

        let res = unsafe { ioctl(self.sock, SIOCGIFMTU, &mut req) };
        if res < 0 {
            return Err(InterfacesError::last_os_error());
        }

        Ok(req.ifr_mtu as u32)
    }
}

fn convert_ifaddr_name(ifa: *mut ffi::ifaddrs) -> Option<String> {
    let ifa = unsafe { &mut *ifa };
    match unsafe { CStr::from_ptr(ifa.ifa_name).to_str() } {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
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
        libc::AF_INET => Kind::Ipv4,
        libc::AF_INET6 => Kind::Ipv6,
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
    let hop = if flags.contains(InterfaceFlags::IFF_BROADCAST) {
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

impl Drop for Interface {
    fn drop(&mut self) {
        let sock = mem::replace(&mut self.sock, 0);
        unsafe { close(sock) };
    }
}

// Helper function
fn copy_slice(dst: &mut [u8], src: &[u8]) -> usize {
    let mut c = 0;

    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = *s;
        c += 1;
    }

    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    #[test]
    fn test_interface_is_comparable() {
        let ifs = Interface::get_all().unwrap();

        assert!(ifs[0] == ifs[0]);
    }

    #[test]
    fn test_hardwareaddr_deriving() {
        let one = HardwareAddr::zero();
        let two = HardwareAddr::zero();

        assert!(one == two);
        assert_is_clone(&one);
        assert_is_copy(&one);
        assert_is_hash(&one);
    }

    #[test]
    fn test_hardwareaddr_format() {
        let h = HardwareAddr::zero();

        assert_eq!(h.as_string(), "00:00:00:00:00:00");
        assert_eq!(h.as_bare_string(), "000000000000");
    }

    fn assert_is_clone<T: Clone>(_: &T) {}
    fn assert_is_copy<T: Copy>(_: &T) {}
    fn assert_is_hash<T: Hash>(_: &T) {}
}
