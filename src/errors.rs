//! Provide error classification and handling for the library.

use std::io;
use std::error::Error;
use std::result;
use std::fmt;
use constants::{MAJOR, MINOR};

/// Report an error in handling a Pico-encoded file.
#[derive(Debug)]
pub enum PicoError {
    /// A file that was expected to exist could not be found.
    FileNotFound(u32, String, io::Error),
    /// A file already exists.
    FileExists(u32, String, io::Error),
    /// Unable to seek to the required location in a file.  Provide a
    /// unique id for the error, and the underlying error from the
    /// io module.
    SeekFailed(u32, io::Error),
    /// Unable to read from a file.  Provide a unique id for the error,
    /// and the underlying error from the io module.
    ReadFailed(u32, io::Error),
    /// Unable to write to a file.  Provide a unique id for the error,
    /// and the underlying error from the io module.
    WriteFailed(u32, io::Error),
    /// The given file is not a Pico-encoded file.  Include the bad magic
    /// number.
    NotPico(u16),
    /// This library cannot handle the Pico-encoded file's version.  Include
    /// the major and minor version numbers of the file.
    BadVersion(u16, u16),
    /// The key has zero length, which is not allowed.
    KeyError,
    /// The specified offset is invalid.  Include the offset value and the
    /// minimum offset value based on the header.
    BadOffset(u32, u32),
    /// An error occurred in computing the hash.
    HashError,
    /// A hrung collapsed somewhere.  Provide a unique id for the error.
    InternalError(u32),
}

impl Error for PicoError {
    fn description(&self) -> &str {
        match *self {
            PicoError::FileNotFound(_, _, _) => r#"File was not found."#,
            PicoError::FileExists(_, _, _) => r#"File already exists."#,
            PicoError::SeekFailed(_, _) => r#"Seeking within a file failed."#,
            PicoError::ReadFailed(_, _) => r#"Reading from a file failed."#,
            PicoError::WriteFailed(_, _) => r#"Writing to a file failed."#,
            PicoError::NotPico(_) => r#"The file does not appear to be a Pico-encoded file."#,
            PicoError::BadVersion(_, _) => r#"This version of the library cannot read the version of the Pico encoding used in the file."#,
            PicoError::KeyError => r#"A key cannot have zero length."#,
            PicoError::BadOffset(_, _) => r#"The data offset in the file is incorrect."#,
            PicoError::HashError => r#"An error occurred computing the hash."#,
            PicoError::InternalError(_) => r#"An internal error was detected in the pico library."#,
        }
    }
    fn cause(&self) -> Option<&Error> {
        Some(match *self {
            PicoError::FileNotFound(_, _, ref err) => err as &Error,
            PicoError::FileExists(_, _, ref err) => err as &Error,
            PicoError::SeekFailed(_, ref err) => err as &Error,
            PicoError::WriteFailed(_, ref err) => err as &Error,
            PicoError::ReadFailed(_, ref err) => err as &Error,
            _ => {
                return None;
            }
        })
    }
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We use the description for display, and then add additional information where appropriate.
        let res = write!(f, "{} ", self.description());
        match *self {
            PicoError::FileNotFound(_, ref name, _) =>
                write!(f, r#"File {:?} was not found."#, name),
            PicoError::FileExists(_, ref name, _) =>
                write!(f, r#"Preventing overwrite of file {:?}, which already exists."#, name),
            PicoError::NotPico(badmagic) =>
                write!(
                    f,
                    r#"First bytes are 0x{:04X} instead of 0x{:04X}, as required."#,
                    badmagic,
                    ::magic()
                ),
            PicoError::BadVersion(badmajor, badminor) =>
                write!(
                    f,
                    r#"This library implements version {}.{} of the Pico encoding, but the file specifies that it uses version {}.{}."#,
                    MAJOR, MINOR, badmajor, badminor
                ),
            PicoError::BadOffset(badoffset, minoffset) =>
                write!(
                    f,
                    r#"The header extends to at least offset 0x{:X}, but the file specifies the data offset as 0x{:X}."#,
                    minoffset, badoffset
                ),
            _ => res,
        }
    }
}

/// A wrapper for easier use of the result type.
pub type Result<T> = result::Result<T, PicoError>;
