extern crate interfaces;

use interfaces::Interface;

fn main() {
    let mut ifs = Interface::get_all().expect("could not get interfaces");
    ifs.sort_by(|a, b| a.name.cmp(&b.name));

    for i in ifs.iter() {
        println!("{}:", i.name);
        println!("----------");
        println!("Flags: {:?}", i.flags);

        println!("Addresses:");
        for addr in i.addresses.iter() {
            println!("- {} - {:?}", addr.kind, addr.addr);
        }
        println!("");
    }

    println!("There are {} interfaces on the system", ifs.len());
}
