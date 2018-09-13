extern crate interfaces2;

use std::env;
use std::process::exit;

use interfaces2::Interface;


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

    let ifname = &args[2];
    let mut i = match Interface::get_by_name(ifname) {
        Ok(Some(i)) => i,
        Ok(None) => {
            println!("Could not find an interface named: {}", ifname);
            return;
        },
        Err(e) => {
            println!("An error occured fetching interfaces: {:?}", e);
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
