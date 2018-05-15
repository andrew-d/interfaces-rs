extern crate interfaces;

use std::iter;
use std::net;

use interfaces::{Interface, Kind, flags::{self, InterfaceFlags}};

// Flag mappings that ifconfig uses, in order.
const NAME_MAPPINGS: &'static [(flags::InterfaceFlags, &'static str)] = &[
    (InterfaceFlags::IFF_UP, "UP"),
    (InterfaceFlags::IFF_LOOPBACK, "LOOPBACK"),
    (InterfaceFlags::IFF_BROADCAST, "BROADCAST"),
    // SMART?
    (InterfaceFlags::IFF_RUNNING, "RUNNING"),
    // SIMPLEX?
    (InterfaceFlags::IFF_MULTICAST, "MULTICAST"),
];

fn main() {
    let mut ifs = Interface::get_all().expect("could not get interfaces");
    ifs.sort_by(|a, b| a.name.cmp(&b.name));

    // Find the maximum alignment for our interface names.
    let max_align = ifs.iter().map(|i| i.name.len() + 2).max().unwrap();

    let full_align = iter::repeat(' ').take(max_align).collect::<String>();

    for i in ifs.iter() {
        let name_align = iter::repeat(' ')
            .take(max_align - i.name.len() - 2)
            .collect::<String>();

        // Build the first line by printing the interface flags.
        let first_line = {
            let mut buf = String::new();
            buf.push_str(&*format!("flags={} <", i.flags.bits()));

            let mut flag_strs = vec![];
            for &(f, s) in NAME_MAPPINGS.iter() {
                if i.flags.contains(f) {
                    flag_strs.push(s);
                }
            }

            buf.push_str(&*flag_strs.join(","));
            buf.push_str(&*format!("> mtu {}", i.get_mtu().unwrap_or(0)));

            buf
        };

        println!("{}: {}{}", i.name, name_align, first_line);

        if i.flags.contains(InterfaceFlags::IFF_LOOPBACK) {
            println!("{}loop (Local Loopback)", full_align);
        } else {
            if let Ok(addr) = i.hardware_addr() {
                println!("{}ether {}", full_align, addr);
            }
        }

        for addr in i.addresses.iter() {
            let raddr = match addr.addr {
                Some(a) => a,
                None => continue,
            };

            let prefix = match addr.kind {
                Kind::Ipv4 => "inet",
                Kind::Ipv6 => "inet6",
                _ => continue,
            };

            println!("{}{} {}", full_align, prefix, format_addr(&raddr));
        }
    }
}

fn format_addr(addr: &net::SocketAddr) -> String {
    match addr {
        &net::SocketAddr::V4(ref a) => format!("{}", a.ip()),
        &net::SocketAddr::V6(ref a) => format!("{}", a.ip()),
    }
}
