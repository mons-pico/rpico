//! Important constants for use in the library.

/// Pico file magic number.
pub const MAGIC: u16 = 0x91c0;

/// Major version number of supported Pico format.
pub const MAJOR: u16 = 1;

/// Minor version number of supported Pico format.
pub const MINOR: u16 = 0;

//
// Field sizes in bytes.
//

/// Size (in bytes) of the magic number.
pub const MAGIC_LEN: usize = 2;

/// Size (in bytes) of the major version number.
pub const MAJOR_LEN: usize = 2;

/// Size (in bytes) of the minor version number.
pub const MINOR_LEN: usize = 2;

/// Size (in bytes) of the data offset.
pub const OFFSET_LEN: usize = 4;

/// Size (in bytes) of the hash.
pub const HASH_LEN: usize = 16;

/// Size (in bytes) of the key length.
pub const KEYLEN_LEN: usize = 2;

//
// Field offsets from start of file.
//

/// Zero-based offset to magic number.
pub const MAGIC_POS: usize = 0;

/// Zero-based offset to major version number.
pub const MAJOR_POS: usize = MAGIC_POS + MAGIC_LEN;

/// Zero-based offset to minor version number.
pub const MINOR_POS: usize = MAJOR_POS + MAJOR_LEN;

/// Zero-based offset to the offset.
pub const OFFSET_POS: usize = MINOR_POS + MINOR_LEN;

/// Zero-based offset to the hash.
pub const HASH_POS: usize = OFFSET_POS + OFFSET_LEN;

/// Zero-based offset to the key length.
pub const KEYLEN_POS: usize = HASH_POS + HASH_LEN;

/// Zero-based offset to the key.
pub const KEY_POS: usize = KEYLEN_POS + KEYLEN_LEN;

/// Desired chunk size to read and write.
pub const CHUNK_SIZE: usize = 4096;
