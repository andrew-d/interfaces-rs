extern crate interfaces;

use std::iter;

use interfaces::{Interface, Kind, flags};


// Flag mappings that ifconfig uses, in order.
const NAME_MAPPINGS: &'static [(flags::InterfaceFlags, &'static str)] = &[
    (flags::IFF_UP, "UP"),
    (flags::IFF_LOOPBACK, "LOOPBACK"),
    (flags::IFF_BROADCAST, "BROADCAST"),
    // SMART?
    (flags::IFF_RUNNING, "RUNNING"),
    // SIMPLEX?
    (flags::IFF_MULTICAST, "MULTICAST"),
];


fn main() {
    let mut ifs = Interface::get_all().expect("could not get interfaces");
    ifs.sort_by(|a, b| a.name.cmp(&b.name));

    // Find the max alignment
    let max_align = ifs.iter()
        .map(|i| i.name.len() + 2)
        .max()
        .unwrap();
    let full_align = iter::repeat(' ')
        .take(max_align)
        .collect::<String>();

    for i in ifs.iter() {
        let name_align = iter::repeat(' ')
            .take(max_align - i.name.len() - 2)
            .collect::<String>();

        let mut first_line = String::new();
        first_line.push_str(&*format!(
            "flags={} <",
            i.flags.bits()
        ));

        let mut flag_strs = vec![];
        for &(f, s) in NAME_MAPPINGS.iter() {
            if i.flags.contains(f) {
                flag_strs.push(s);
            }
        }

        first_line.push_str(&*flag_strs.join(","));
        first_line.push_str(&*format!(
            "> mtu {}",
            i.get_mtu().unwrap_or(0)
        ));

        println!("{}: {}{}", i.name, name_align, first_line);

        if !i.flags.contains(flags::IFF_LOOPBACK) {
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

            println!("{}{} {}", full_align, prefix, raddr);
        }
    }
}
