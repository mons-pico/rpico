//! Function to encrypt / decrypt data in the Pico file.

/// Encrypt or decrypt, in place, the given data.
///
/// Because Pico uses simple symmetric xor encryption, this method will
/// both encrypt and decrypt.
///
/// # Arguments
/// * `position` - Zero-based position where the data will reside in the file.
/// * `data`     - The data to encrypt or decrypt.
/// * `key`      - The key to use for encryption and decryption.
pub fn crypt(position: usize, data: &mut [u8], key: &Vec<u8>) {
    let klen = key.len() as usize;
    for index in 0..(data.len()) {
        data[index] ^= key[(index + position) % klen];
    }
}
