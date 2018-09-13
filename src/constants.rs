use std::collections::HashMap;
use std::convert::AsRef;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
use std::ptr;

#[cfg(all(not(target_pointer_width = "32"), not(target_env = "musl"), not(target_os="android")))]
pub type ConstantType = u64;

#[cfg(all(target_pointer_width = "32", not(target_env = "musl"), not(target_os="android")))]
pub type ConstantType = u32;

#[cfg(any(target_env = "musl", target_os="android") )]
pub type ConstantType = i32;


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
        let ret = cvals
            .into_iter()
            .map(|v| {
                // HashMap has a from_iter method that accepts (key, value) tuples.
                 (
                     unsafe { CStr::from_ptr(v.name).to_string_lossy().into_owned() },
                     v.value as ConstantType
                 )
            })
            .collect::<HashMap<_, _>>();
        ret
    };
}

pub fn get_constant<S: AsRef<str>>(name: S) -> Option<ConstantType> {
    // Since `u64` is `Copy`, we can dereference the constant directly
    CONSTANTS.get(name.as_ref()).map(|v| *v)
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

