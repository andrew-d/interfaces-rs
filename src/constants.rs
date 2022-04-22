use std::collections::HashMap;
use std::convert::AsRef;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
use std::ptr;

use lazy_static::lazy_static;

#[cfg(any(
    target_os = "fuchsia",
    target_os = "haiku",
    target_os = "hermit",
    target_os = "android",
    target_os = "emscripten",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "vxworks",
    target_os = "wasi",
    target_env = "wasi",
))]
pub type ConstantType = libc::c_int;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "redox",
    target_env = "newlib",
    target_env = "uclibc",
))]
pub type ConstantType = libc::c_ulong;

#[cfg(all(target_os = "linux", target_env = "musl"))]
pub type ConstantType = libc::c_int;

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub type ConstantType = libc::c_ulong;

/// The constant as sent by the C side.
#[repr(C)]
struct Constant {
    name: *const c_char,
    value: u64,
}

extern "C" {
    /// Get a list of all constants compiled into the C code.
    fn rust_get_constants() -> *const Constant;
}

lazy_static! {
    static ref CONSTANTS: HashMap<String, ConstantType> = {
        let mut cvals = vec![];

        let mut constant = unsafe { rust_get_constants() };

        // The C function will return a static array that contains the various constants we requested,
        // terminated by a single entry in the array that has a NULL pointer for a name.  We loop
        // forever until we see this entry.
        loop {
            let cval = unsafe { ptr::read(constant) };
            if cval.name.is_null() {
                break
            }

            // Save this entry
            cvals.push(cval);

            // Bump pointer.  This "steps" through the array by the size of the underlying
            // structure.
            constant = ((constant as usize) + mem::size_of::<Constant>()) as *const Constant;
        }

        // Convert from the C-provided type into a hashmap.
        cvals
            .into_iter()
            .map(|v| {
                // HashMap has a from_iter method that accepts (key, value) tuples.
                 (
                     unsafe { CStr::from_ptr(v.name).to_string_lossy().into_owned() },
                     v.value as ConstantType
                 )
            })
            .collect::<HashMap<_, _>>()
    };
}

pub fn get_constant<S: AsRef<str>>(name: S) -> Option<ConstantType> {
    // Since `u64` is `Copy`, we can dereference the constant directly
    CONSTANTS.get(name.as_ref()).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_existing() {
        assert!(get_constant("SIOCGIFFLAGS").is_some())
    }

    #[test]
    fn test_not_existing() {
        assert!(get_constant("bad key").is_none())
    }
}
