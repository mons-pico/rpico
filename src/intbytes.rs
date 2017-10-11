//! Handle mapping unsigned integers to bytes and back.
use std::io;

/// A trait to convert integer types to byte arrays.
pub trait ByteDump {
    /// Get an array of bytes, in big-endian order, for the type.
    fn get_bytes(&self) -> Box<[u8]>;

    /// Write the bytes, in big endian order, to the provided stream.
    /// Numbers can be printed in hexadecimal or decimal.  If printed
    /// in hexadecimal, then a leading `0x` is appended.  The output
    /// is comma-separated.
    ///
    /// For instance, 7u32 becomes `0x00, 0x00, 0x00, 0x07`.
    ///
    /// # Arguments
    /// * `target` - The stream to get the output.
    /// * `hex`    - If true, print the numbers in hexadecmial.
    #[allow(unused_must_use)]
    fn dump_bytes<U: io::Write>(&self, target: &mut U, hex: bool) {
        let bytes = self.get_bytes();
        let mut first = true;
        for byte in bytes.iter() {
            if first {
                first = false;
            } else {
                write!(target, ", ");
            }
            if hex {
                write!(target, "{:#04X}", byte);
            } else {
                write!(target, "{}", byte);
            }
        } // Print the bytes.
    }
}

impl ByteDump for u8 {
    fn get_bytes(&self) -> Box<[u8]> {
        let arr: [u8; 1] = [*self as u8];
        Box::new(arr)
    }
}

impl ByteDump for u16 {
    fn get_bytes(&self) -> Box<[u8]> {
        let arr: [u8; 2] = [(*self >> 8) as u8, *self as u8];
        Box::new(arr)
    }
}

impl ByteDump for u32 {
    fn get_bytes(&self) -> Box<[u8]> {
        let arr: [u8; 4] = [
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ];
        Box::new(arr)
    }
}

impl ByteDump for u64 {
    fn get_bytes(&self) -> Box<[u8]> {
        let arr: [u8; 8] = [
            (*self >> 56) as u8,
            (*self >> 48) as u8,
            (*self >> 40) as u8,
            (*self >> 32) as u8,
            (*self >> 24) as u8,
            (*self >> 16) as u8,
            (*self >> 8) as u8,
            *self as u8,
        ];
        Box::new(arr)
    }
}

/// Dump a vector to the given stream as text.
///
/// The vector consists of a sequence of bytes that can be written in
/// either decimal or hexadecimal, and can be separated with commas.
///
/// For instance, `vec![7u8, 9u8, 210u8]` becomes
/// `0x07, 0x09, 0xD2` if both `hex` and `commas` are true.
///
/// Note that commas are always used for decimal output.
///
/// # Arguments
/// * `target` - The stream to get the output.
/// * `hex`    - If true, print the numbers in hexadecmial.
/// * `commas` - If true, print commas between numbers.
#[allow(unused_must_use)]
pub fn dump_vec<U: io::Write>(target: &mut U, bytes: &Vec<u8>, hex: bool, commas: bool) {
    let mut first = true;
    for byte in bytes {
        if (!hex) || commas {
            if first {
                first = false;
            } else {
                write!(target, ", ");
            }
        }
        if hex {
            if commas {
                write!(target, "{:#04X}", byte);
            } else {
                write!(target, "{:02X}", byte);
            }
        } else {
            write!(target, "{}", byte);
        }
    }
}

#[allow(unused_imports)]
mod test {
    // These imports are needed, but the compiler thinks they are not.
    use super::ByteDump;
    use super::dump_vec;

    #[test]
    #[inline]
    fn dump_u8() {
        let mut output: Vec<u8> = Vec::new();
        (0x49 as u8).dump_bytes(&mut output, true);
        assert_eq!(output, Vec::<u8>::from("0x49"));
    }

    #[test]
    #[inline]
    fn dump_u16() {
        let mut output: Vec<u8> = Vec::new();
        (0x7c84 as u16).dump_bytes(&mut output, true);
        assert_eq!(output, Vec::<u8>::from("0x7C, 0x84"));
    }

    #[test]
    #[inline]
    fn dump_u32() {
        let mut output: Vec<u8> = Vec::new();
        (0x4acba5b as u32).dump_bytes(&mut output, true);
        assert_eq!(output, Vec::<u8>::from("0x04, 0xAC, 0xBA, 0x5B"));
    }

    #[test]
    #[inline]
    fn dump_u64() {
        let mut output: Vec<u8> = Vec::new();
        (0x04ACBA5B0055ff23 as u64).dump_bytes(&mut output, true);
        assert_eq!(
            output,
            Vec::<u8>::from("0x04, 0xAC, 0xBA, 0x5B, 0x00, 0x55, 0xFF, 0x23")
        );
    }

    #[test]
    #[inline]
    fn dump_vec_test() {
        let mut output: Vec<u8> = Vec::new();
        let value = vec![0x21, 0x56, 0xff, 0x32, 0x18];
        dump_vec(&mut output, &value, true, true);
        assert_eq!(output, Vec::<u8>::from("0x21, 0x56, 0xFF, 0x32, 0x18"));
        let mut output: Vec<u8> = Vec::new();
        dump_vec(&mut output, &value, true, false);
        assert_eq!(output, Vec::<u8>::from("2156FF3218"));
    }
}
