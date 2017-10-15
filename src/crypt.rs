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
    if klen == 0 { panic!("Zero length key."); }
    for index in 0..(data.len()) {
        data[index] ^= key[(index + position) % klen];
    }
}

mod test {
    use super::crypt;

    #[test]
    fn crypt_test_1() {
        let key = vec![0u8];
        let mut data = [18u8, 21u8];
        crypt(0, &mut data, &key);
        assert_eq!(data, [18u8, 21u8]);
    }

    #[test]
    fn crypt_test_2() {
        let key = vec![0u8];
        let mut data: [u8;0] = [];
        crypt(0, &mut data, &key);
        assert_eq!(data, []);
    }

    #[test]
    fn crypt_test_3() {
        let key = vec![0x40u8, 0x09u8];
        let mut data = [0x09u8, 0x20u8, 0x00u8, 0xe0u8];
        crypt(0, &mut data, &key);
        assert_eq!(data, [0x49u8, 0x29u8, 0x40u8, 0xe9u8]);
    }

    #[test]
    fn crypt_test_4() {
        let key = vec![0xaau8, 0x55u8, 0x63u8, 0xf7u8, 0x7eu8];
        let mut data = [0x9au8, 0xd4u8, 0x6cu8, 0x58u8];
        crypt(0, &mut data, &key);
        assert_eq!(data, [0x30u8, 0x81u8, 0x0fu8, 0xafu8]);
    }
}
