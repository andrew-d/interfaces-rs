use std::convert::From;
use std::error::Error;
use std::fmt;

/// InterfacesError is the error type that is returned by all functions in this crate.  See the
/// documentation on the individual variants for more information.
#[derive(Debug)]
pub enum InterfacesError {
    /// Errno indicates that something went wrong with an underlying syscall.  The internal value
    /// is the `errno` that was returned.
    Errno(nix::errno::Errno),

    /// NotSupported indicates that something required for this operation is not currently
    /// supported on this platform or computer.  The internal string may contain more detail.
    NotSupported(&'static str),
}

impl InterfacesError {
    /// Create a new instance of `InterfacesError` with the error set to the current value of the
    /// libc `errno` variable.
    pub fn last_os_error() -> InterfacesError {
        InterfacesError::Errno(nix::errno::Errno::last())
    }
}

impl From<nix::errno::Errno> for InterfacesError {
    fn from(e: nix::errno::Errno) -> InterfacesError {
        InterfacesError::Errno(e)
    }
}

impl Error for InterfacesError {
    fn description(&self) -> &str {
        use InterfacesError::*;

        match *self {
            Errno(..) => "A syscall error occured",
            NotSupported(..) => "A required feature is not supported",
        }
    }
}

impl fmt::Display for InterfacesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InterfacesError::*;

        match *self {
            Errno(ref err) => write!(f, "Errno({})", err.desc()),
            NotSupported(msg) => write!(f, "NotSupported({})", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fmt;

    use super::*;

    #[test]
    fn test_error_has_traits() {
        let e = InterfacesError::last_os_error();

        assert_is_error(&e);
        assert_is_display(&e);
    }

    fn assert_is_error<T: Error>(_: &T) {}
    fn assert_is_display<T: fmt::Display>(_: &T) {}
}
