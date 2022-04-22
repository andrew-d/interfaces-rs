use std::convert::From;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::exit;

use std::env;

use handlebars as hbs;
use serde::{Deserialize, Serialize};

fn main() {
    let in_path = Path::new("src").join("constants.c.in");
    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("constants.c");

    // Template the file.
    if let Err(e) = template_file(&in_path, &out_path) {
        println!("Error creating `constants.c` from template");
        println!("-> {:?}", e);
        exit(1);
    }

    // Build the final library
    let mut cfg = cc::Build::new();

    let helpers_path = Path::new("src").join("helpers.c");
    let ifaddrs_path = Path::new("src").join("ifaddrs.c");

    let cfg = cfg.file(&out_path).file(&helpers_path);

    if env::var_os("CARGO_CFG_TARGET_OS")
        .unwrap()
        .to_str()
        .unwrap()
        == "android"
    {
        cfg.file(ifaddrs_path);
    }

    cfg.compile("libinterfaces.a");
}

fn template_file(in_path: &PathBuf, out_path: &PathBuf) -> Result<(), Error> {
    // Open and read the file.
    let mut f = File::open(in_path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    let mut handlebars = hbs::Handlebars::new();
    handlebars.register_template_string("template", s)?;

    let mut f = File::create(out_path)?;

    let data = make_data();
    handlebars.renderw("template", &data, &mut f)?;

    Ok(())
}

fn make_data() -> Context {
    // These constants are "dynamically" generated by compiling a C file that includes their value
    // and then including that in the final build.  See `constants.rs` for a function that can be
    // used to retrieve them.
    let names: &[&str] = &[
        // IOCTLs
        "SIOCGIFCONF",
        "SIOCGIFHWADDR",
        "SIOCGIFFLAGS",
        "SIOCSIFFLAGS",
        "SIOCGIFMTU",
        "SIOCSIFMTU",
        // Address families
        "AF_LINK",
        "AF_PACKET", // Only on Linux
    ];

    // These constants are the same as above, but we don't test them for existence with #ifdef.
    let anames: &[&str] = &["sizeof(struct ifreq)"];

    let names = names
        .iter()
        .map(|x| String::from(*x))
        .collect::<Vec<String>>();
    let anames = anames
        .iter()
        .map(|x| String::from(*x))
        .collect::<Vec<String>>();

    Context {
        test_constants: names,
        always_constants: anames,
    }
}

#[derive(Serialize, Deserialize)]
struct Context {
    test_constants: Vec<String>,
    always_constants: Vec<String>,
}

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Template(hbs::TemplateError),
    Render(hbs::RenderError),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<hbs::TemplateError> for Error {
    fn from(e: hbs::TemplateError) -> Error {
        Error::Template(e)
    }
}

impl From<hbs::RenderError> for Error {
    fn from(e: hbs::RenderError) -> Error {
        Error::Render(e)
    }
}
