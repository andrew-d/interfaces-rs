use std::convert::From;

use nix;

/// InterfacesError is the error type that is returned by all functions in this crate.  See the
/// documentation on the individual variants for more information.
#[derive(Debug)]
pub enum InterfacesError {
    /// Errno indicates that something went wrong with an underlying syscall.  The internal value
    /// is the `errno` that was returned.
    Errno(nix::errno::Errno),
}

impl InterfacesError {
    pub fn last_errno() -> InterfacesError {
        return InterfacesError::Errno(nix::errno::Errno::last());
    }
}

impl From<nix::errno::Errno> for InterfacesError {
    fn from(e: nix::errno::Errno) -> InterfacesError {
        InterfacesError::Errno(e)
    }
}
