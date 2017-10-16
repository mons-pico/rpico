//! Rust library for Pico encoding.
//!
//! This is a library implementing the Pico file encoding used for storing
//! malware.  See http://mons-pico.github.io/ for details on this.

extern crate md5;

#[warn(missing_docs)]

pub mod constants;
pub mod errors;
mod pico;
pub mod file;
mod crypt;
mod intbytes;
mod header;
pub use pico::Pico;
pub use header::HeaderFormat;
use constants::{MAGIC, MINOR, MAJOR};

/// Obtain the Pico magic number.  The "magic number" used at the start of a
/// file to indicate that it is a Pico-encoded file.
///
/// ```
/// println!("The magic number is {:#04X}", pico::magic());
/// ```
pub fn magic() -> u16 { MAGIC }

/// Obtain the major version number for the encoding implemented by this
/// library.  See also `minor`.
pub fn major() -> u16 { MAJOR }

/// Obtain the minor version number for the encoding implemented by this
/// library.  See also `major`.
pub fn minor() -> u16 { MINOR }

#[test]
fn check_version() {
    assert_eq!(major(), MAJOR);
    assert_eq!(minor(), MINOR);
}

#[test]
fn check_magic() {
    assert_eq!(magic(), MAGIC);
}
