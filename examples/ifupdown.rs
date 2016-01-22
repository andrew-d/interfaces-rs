extern crate interfaces;

use std::env;
use std::process::exit;

use interfaces::Interface;


fn main() {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 3 {
        usage();
    }
    let new_status = match &*args[1] {
        "up" => true,
        "down" => false,
        _ => usage(),
    };

    let ifs = match Interface::get_all() {
        Ok(ifs) => ifs,
        Err(e) => {
            println!("Could not get interfaces: {:?}", e);
            return;
        },
    };

    let ifname = &args[2];
    let mut i = match ifs.into_iter().find(|x| &x.name == ifname) {
        Some(i) => i,
        None => {
            println!("Could not find an interface named: {}", ifname);
            return;
        },
    };

    println!("Interface {} was {}", i.name, if i.is_up() { "up" } else { "down" });
    match i.set_up(new_status) {
        Ok(_) => {
            println!("Successfully set interface status");
            println!("Interface is now {}", if new_status { "up" } else { "down" });
        },
        Err(e) => {
            println!("Could not set interface status: {:?}", e);
        },
    };
}

fn usage() -> ! {
    println!("Usage: {} up/down <interface>", env::args().next().unwrap());
    exit(1);
}
