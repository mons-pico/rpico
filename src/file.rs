//! File operations and core data structure for the Pico library.
//!
//! # Pico File Layout
//!
//! ^ Offset      ^ Data  ^ Meaning ^ Code ^
//! | 0x00 - 0x01 | Magic Number ||
//! | 0x02 - 0x03 | Major Version Number | `get_major()` |
//! | 0x04 - 0x05 | Minor Version Number | `get_minor()` |
//! | 0x06 - 0x09 | Data Offset | `get_offset()` |
//! | 0x0A - 0x19 | MD5 Byte 16 | `get_hash()` |
//! | 0x1A - 0x1B | Key Length High Byte | `get_key().len()` |
//! | 0x1C - | Start of Key Bytes | `get_key()` |
//!
//! The end of the key is the start of the metadata, if any.
//! The length of the metadata section is given by
//! `get_md_length()`.
//!
//! The offset points to the first byte of the data, which
//! immediately follows the metadata, or the key if there is
//! no metadata.

use std::io::{Read, Write, Seek, SeekFrom};
use header::HeaderFormat;
use constants::*;
use crypt::crypt;
use intbytes::{ByteDump, dump_vec};
use errors::{PicoError, Result};
use md5;

/// Wrapper to handle Pico encoding and decoding.
///
/// # Use
/// You may use this to create a new file via the `new` method, or you
/// can work with an existing file via the `open` method.  In both cases
/// you need to actually open the file, first.
///
/// ## Metadata
/// You can access metadata via the `put_metadata` and `get_metadata`
/// methods, provided you have allocated space for metadata in the file.
/// Check `get_md_length` to find out the number of bytes reserved for
/// metadata.
///
/// ## Data
/// Data is read and decrypted via the `get_data` method, and data is
/// encrypted and written via the `put_data` method.  Limits on data size
/// are controlled by the underlying file system (and the `usize` type).
pub struct Pico<T: Seek + Read + Write> {
    /// Major version number in the file.
    major: u16,
    /// MinMinorTion number in the file.
    minor: u16,
    /// Zero-based offset to the start of data.
    offset: u32,
    /// The hash.
    hash: [u8; HASH_LEN],
    /// The key length.
    key: Vec<u8>,
    /// Whether the hash is valid.
    is_hash_valid: bool,
    /// Zero-based start of metadata.
    md_start: usize,
    /// The metadata length in bytes.
    md_length: usize,
    /// The header owns the file.
    file: T,
}

impl<T: Seek + Read + Write> Pico<T> {
    /// Get the version number of the encoding used to create this file.
    pub fn get_version(&self) -> (u16, u16) {
        (self.major, self.minor)
    }

    /// Get the zero-based offset within the file of the first data byte.
    pub fn get_offset(&self) -> u32 {
        self.offset
    }

    /// Get the hash value of the data in the file.  Note that this may
    /// cause the hash to be computed if it is not already valid.
    pub fn get_hash(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    /// Get the encryption key used to encrypt the data in this file.
    pub fn get_key(&self) -> Vec<u8> {
        self.key.clone()
    }

    /// Dump the content of the header in the correct form.
    ///
    /// # Arguments
    /// * `target` - The writer to get the output.
    /// * `form`   - The format to use to write.
    #[allow(unused_must_use)]
    pub fn dump_header<U>(&self, target: &mut U, form: HeaderFormat)
    where
        U: Write,
    {
        let (major, minor) = self.get_version();
        let hash = self.get_hash();
        let key = self.get_key();
        match form {
            HeaderFormat::DICT => {
                write!(target, "{{\n");
                write!(target, "    \"magic\" : [ ");
                ::magic().dump_bytes(target, true);
                write!(target, " ],\n");
                write!(target, "    \"major\" : {},\n", major);
                write!(target, "    \"minor\" : {},\n", minor);
                write!(target, "    \"offset\" : {},\n", self.get_offset());
                write!(target, "    \"hash\" : [ ");
                dump_vec(target, &hash, true, true);
                write!(target, " ],\n");
                write!(target, "    \"key_length\" : {},\n", key.len());
                write!(target, "    \"key\" : [ ");
                dump_vec(target, &key, true, true);
                write!(target, " ],\n");
                write!(target, "    \"md_length\" : {},\n", self.get_md_length());
                write!(target, "}}\n");
            }
            HeaderFormat::JSON => {
                write!(target, "{{\n");
                write!(target, "    \"magic\" : [ ");
                ::magic().dump_bytes(target, false);
                write!(target, " ],\n");
                write!(target, "    \"major\" : {},\n", major);
                write!(target, "    \"minor\" : {},\n", minor);
                write!(target, "    \"offset\" : {},\n", self.get_offset());
                write!(target, "    \"hash\" : [ ");
                dump_vec(target, &hash, false, true);
                write!(target, " ],\n");
                write!(target, "    \"key_length\" : {},\n", key.len());
                write!(target, "    \"key\" : [ ");
                dump_vec(target, &key, false, true);
                write!(target, " ],\n");
                write!(target, "    \"md_length\" : {},\n", self.get_md_length());
                write!(target, "}}\n");
            }
            HeaderFormat::YAML => {
                write!(target, "magic: [ ");
                ::magic().dump_bytes(target, false);
                write!(target, " ]\n");
                write!(target, "major: {}\n", major);
                write!(target, "minor: {}\n", minor);
                write!(target, "offset: {}\n", self.get_offset());
                write!(target, "hash: [ ");
                dump_vec(target, &hash, false, true);
                write!(target, " ]\n");
                write!(target, "key_length: {}\n", key.len());
                write!(target, "key: [ ");
                dump_vec(target, &key, false, true);
                write!(target, " ]\n");
                write!(target, "md_length: {}\n", self.get_md_length());
            }
            HeaderFormat::XML => {
                write!(
                    target,
                    r#"<pico magic='0x{:04X}' major='{}' minor='{}' offset='{}'"#,
                    ::magic(),
                    major,
                    minor,
                    self.get_offset()
                );
                write!(target, " hash='");
                dump_vec(target, &hash, true, false);
                write!(target, " key='");
                dump_vec(target, &key, true, false);
                write!(target, " md_length='{}' />", self.get_md_length());
            }
        }
    }

    /// Create a new Pico-encoded file.
    ///
    /// # Arguments
    /// * `file`      - An open file for writing that must support `seek`.
    /// * `key`       - The encryption key to use.  If this is empty, a random key
    ///                 is generated.
    /// * `md_length` - The number of bytes to reserve for metadata.  Can be zero.
    pub fn new(file: T, key: Vec<u8>, md_length: u32) -> Result<Pico<T>>
    where
        T: Read + Write + Seek,
    {
        let md_start = key.len() + KEY_POS;
        let mut pico = Pico {
            major: MAJOR,
            minor: MINOR,
            offset: md_start as u32 + md_length,
            hash: [0; HASH_LEN as usize],
            is_hash_valid: false,
            key: key,
            md_start: md_start,
            md_length: md_length as usize,
            file: file,
        };
        pico.write_header()?;
        pico.file.flush().map_err(
            |err| PicoError::WriteFailed(1001, err),
        )?;
        Ok(pico)
    }

    /// Initialize from an existing, open, Pico-encoded file.
    pub fn open(mut file: T) -> Result<Pico<T>>
    where
        T: Read + Write + Seek,
    {
        // Allocate some buffers.
        let mut u16buf = [0u8; 2];
        let mut u32buf = [0u8; 4];

        // Little functions to assemble types from the buffers.
        fn tou16(buf: [u8; 2]) -> u16 {
            ((buf[0] as u16) << 8) | (buf[1] as u16)
        }
        fn tou32(buf: [u8; 4]) -> u32 {
            ((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) | ((buf[2] as u32) << 8) |
                (buf[3] as u32)
        }

        // Read the magic number from the file.
        file.read(&mut u16buf).map_err(
            |err| PicoError::ReadFailed(1002, err),
        )?;
        let magic = tou16(u16buf);
        if magic != ::magic() {
            return Err(PicoError::NotPico(magic));
        }

        // Read the version numbers from the file.
        file.read(&mut u16buf).map_err(
            |err| PicoError::ReadFailed(1003, err),
        )?;
        let major = tou16(u16buf);
        file.read(&mut u16buf).map_err(
            |err| PicoError::ReadFailed(1004, err),
        )?;
        let minor = tou16(u16buf);
        if major > MAJOR {
            return Err(PicoError::BadVersion(major, minor));
        }

        // Read the offset from the file.
        file.read(&mut u32buf).map_err(
            |err| PicoError::ReadFailed(1005, err),
        )?;
        let offset = tou32(u32buf);

        // Read the hash from the file.
        let mut hash = [0u8; HASH_LEN];
        file.read(&mut hash).map_err(
            |err| PicoError::ReadFailed(1006, err),
        )?;

        // Read the key length from the file.
        file.read(&mut u16buf).map_err(
            |err| PicoError::ReadFailed(1007, err),
        )?;
        let keylen = tou16(u16buf);
        if keylen == 0 {
            return Err(PicoError::KeyError);
        }

        // Read the key.
        let mut key = vec![0u8; keylen as usize];
        file.read(&mut key).map_err(|err| {
            PicoError::ReadFailed(1008, err)
        })?;

        // Compute the metadata start and length.
        let md_start = KEY_POS + keylen as usize;
        if (offset as usize) < md_start {
            return Err(PicoError::BadOffset(offset, md_start as u32));
        }
        let md_length = offset as usize - md_start;

        // Done.
        Ok(Pico {
            major: major,
            minor: minor,
            offset: offset,
            key: key,
            hash: hash,
            md_length: md_length,
            md_start: md_start,
            is_hash_valid: true,
            file: file,
        })
    }

    /// Write everything to the file.  This may force computation of the hash.
    pub fn flush(&mut self) -> Result<()> {
        self.check_hash()?;
        self.write_header()?;
        self.file.flush().map_err(
            |err| PicoError::WriteFailed(1009, err),
        )?;
        Ok(())
    }

    /// Get the number of bytes reserved for metadata.
    pub fn get_md_length(&self) -> u32 {
        self.md_length as u32
    }

    /// Obtain some part of the stored metadata.
    ///
    /// # Arguments
    /// * `start`  - Zero-based start index within the metadata.
    /// * `buffer` - The buffer to get the metadata.
    ///
    /// If possible, the buffer is filled.  The number of bytes read is
    /// returned.
    pub fn get_metadata(&mut self, start: u32, buffer: &mut [u8]) -> Result<usize> {
        // If there is no metadata, then stop now.  If the offset is past the end
        // of the metadata, then stop now.
        let mdlen = self.get_md_length();
        if mdlen == 0 || start >= mdlen { return Ok(0); }

        // Compute the true offset to the metadata.
        let true_offset = start as usize + self.md_start;

        // Compute the maximum number of bytes that can be read from the
        // metadata, and then figure out how many bytes we actually need to
        // read.
        let mut max = (mdlen - start) as usize;
        if max > buffer.len() {
            max = buffer.len();
        }

        // Seek to the true offset.
        self.file
            .seek(SeekFrom::Start(true_offset as u64))
            .map_err(|err| PicoError::SeekFailed(1010, err))?;

        // Read the requested number of bytes from the metadata.
        let count = self.file.read(&mut buffer[0..max]).map_err(|err| {
            PicoError::ReadFailed(1011, err)
        })?;

        // Success.
        Ok(count)
    }

    /// Write into the metadata section.
    ///
    /// # Arguments
    /// * `start`  - Zero-based start index within the metadata.
    /// * `buffer` - The metadata to write.
    ///
    /// If possible, the entire buffer is written.  This may not be the case
    /// if there is insufficient room for the data.  The number of bytes written
    /// is returned.
    pub fn put_metadata(&mut self, start: u32, buffer: &[u8]) -> Result<usize> {
        // If there is no metadata, then stop now.  If the offset is past the end
        // of the metadata, then stop now.
        let mdlen = self.get_md_length();
        if mdlen == 0 || start >= mdlen { return Ok(0); }

        // Compute the true offset to the metadata.
        let true_offset = start as usize + self.md_start;

        // Compute the maximum number of bytes that can be written to the
        // metadata, and then figure out how many bytes we actually need to
        // write.
        let mut max = (mdlen - start) as usize;
        if max > buffer.len() {
            max = buffer.len();
        }

        // If nothing to write, stop now.
        if max <= 0 {
            return Ok(0);
        }

        // Seek to the true offset.
        self.file
            .seek(SeekFrom::Start(true_offset as u64))
            .map_err(|err| PicoError::SeekFailed(1012, err))?;

        // Write the requested number of bytes to the metadata.
        let count = self.file.write(&buffer[0..max]).map_err(|err| {
            PicoError::WriteFailed(1013, err)
        })?;

        // Success.
        Ok(count)
    }

    /// Get raw, unencrypted data from the file.
    ///
    /// # Arguments
    /// * `position` - Zero-based index within the data.
    /// * `buffer`   - The buffer to get the data.
    ///
    /// If possible, the buffer is filled.  The number of bytes read is
    /// returned.
    pub fn get(&mut self, position: usize, buffer: &mut [u8]) -> Result<usize> {
        // Compute the true offset to the data.
        let true_offset = position + self.get_offset() as usize;

        // Seek to the true offset.
        self.file
            .seek(SeekFrom::Start(true_offset as u64))
            .map_err(|err| PicoError::SeekFailed(1014, err))?;

        // Read the requested number of bytes from the data.
        let count = self.file.read(buffer).map_err(
            |err| PicoError::ReadFailed(1015, err),
        )?;

        // Decrypt the data received.
        let key = self.get_key();
        crypt(position, buffer, &key);

        // Success.
        Ok(count)
    }

    /// Encrypt and store the given data in the file.  Note that the data is
    /// encrypted in place, so the buffer is modified by this method.
    ///
    /// # Arguments
    /// * `position` - Zero-based index within the data.
    /// * `buffer`   - The data to encrypt and write.
    ///
    /// If possible, the entire buffer is written.  The number of bytes written
    /// is returned.
    pub fn put(&mut self, position: usize, buffer: &mut [u8]) -> Result<usize> {
        // Compute the true offset to the data.
        let true_offset = position + self.get_offset() as usize;

        // Seek to the true offset.
        self.file
            .seek(SeekFrom::Start(true_offset as u64))
            .map_err(|err| PicoError::SeekFailed(1016, err))?;

        // Encrypt the data to be sent.
        let key = self.get_key();
        crypt(position, buffer, &key);

        // Write the requested number of bytes to the data.
        let count = self.file.write(buffer).map_err(|err| {
            PicoError::ReadFailed(1017, err)
        })?;

        // Success.
        Ok(count)
    }

    fn check_hash(&mut self) -> Result<()> {
        // If the hash is valid, there is nothing to do.
        if self.is_hash_valid {
            return Ok(());
        }

        // Re-compute the hash.  To do that, read back through the entire
        // data segment, decrypt it, compute the hash, and write it to the
        // stored header.

        let mut position: usize = 0;
        let mut buffer: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let mut context = md5::Context::new();
        loop {
            let num = self.get(position, &mut buffer)?;
            if num == 0 {
                break;
            }
            context.consume(&buffer[..]);
            position = position + num;
        }
        self.hash = *context.compute();
        self.is_hash_valid = true;
        Ok(())
    }

    fn write_header(&mut self) -> Result<()> {
        // Seek to the start of the file.
        self.file.seek(SeekFrom::Start(0)).map_err(|err| {
            PicoError::SeekFailed(1018, err)
        })?;

        // Write the magic number.
        let item = ::magic().get_bytes();
        self.file.write(&item).map_err(|err| {
            PicoError::WriteFailed(1019, err)
        })?;

        // Write the version number.
        let (major, minor) = self.get_version();
        let item = major.get_bytes();
        self.file.write(&item).map_err(|err| {
            PicoError::WriteFailed(1020, err)
        })?;
        let item = minor.get_bytes();
        self.file.write(&item).map_err(|err| {
            PicoError::WriteFailed(1021, err)
        })?;

        // Write the offset to the data.
        let item = self.get_offset().get_bytes();
        self.file.write(&item).map_err(|err| {
            PicoError::WriteFailed(1022, err)
        })?;

        // Write the hash.
        let item = self.get_hash();
        self.file.write(item.as_slice()).map_err(|err| {
            PicoError::WriteFailed(1023, err)
        })?;

        // Write the key length and then the key.
        let key = self.get_key();
        let item = (key.len() as u16).get_bytes();
        self.file.write(&item).map_err(|err| {
            PicoError::WriteFailed(1024, err)
        })?;
        self.file.write(key.as_slice()).map_err(|err| {
            PicoError::WriteFailed(1025, err)
        })?;

        // If we get here, success!
        Ok(())
    }
}

#[allow(unused_imports)]
mod test {
    use std::fs::OpenOptions;
    use std::fs::create_dir_all;
    use std::fs::remove_file;
    use super::Pico;

    #[test]
    fn hash_test() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/hash_test.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        pico.check_hash().unwrap();
        assert_eq!(pico.get_hash(), vec![
            0xd4u8, 0x1du8, 0x8cu8, 0xd9u8,
            0x8fu8, 0x00u8, 0xb2u8, 0x04u8,
            0xe9u8, 0x80u8, 0x09u8, 0x98u8,
            0xecu8, 0xf8u8, 0x42u8, 0x7eu8]);
        remove_file("_test/hash_test.pico").unwrap();
    }

    #[test]
    fn md_test_1() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/md_test_1.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        assert_eq!(pico.put_metadata(0, b"Martindale").unwrap(), 10);
        let mut data = [0u8; 10];
        assert_eq!(pico.get_metadata(0, &mut data).unwrap(), 10);
        assert_eq!(&data, b"Martindale");
        remove_file("_test/md_test_1.pico").unwrap();
    }

    #[test]
    fn md_test_2() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/md_test_2.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        assert_eq!(pico.put_metadata(0, b"Martindale").unwrap(), 10);
        let mut data = [0u8; 10];
        assert_eq!(pico.get_metadata(5, &mut data).unwrap(), 5);
        assert_eq!(&data, b"ndale\0\0\0\0\0");
        remove_file("_test/md_test_2.pico").unwrap();
    }

    #[test]
    fn md_test_3() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/md_test_3.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        assert_eq!(pico.put_metadata(5, b"Martindale").unwrap(), 5);
        let mut data = [0u8; 10];
        assert_eq!(pico.get_metadata(0, &mut data).unwrap(), 10);
        assert_eq!(&data, b"\0\0\0\0\0Marti");
        remove_file("_test/md_test_3.pico").unwrap();
    }

    #[test]
    fn md_test_4() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/md_test_4.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        assert_eq!(pico.put_metadata(0, b"Martindal").unwrap(), 9);
        let mut data = [0u8; 10];
        // The result here might be 10, or it might be 9.  It depends on whether or
        // not the metadata has been padded out.  We don't actually care, do we don't
        // check.
        pico.get_metadata(0, &mut data).unwrap();
        assert_eq!(&data, b"Martindal\0");
        remove_file("_test/md_test_4.pico").unwrap();
    }

    #[test]
    fn md_test_5() {
        create_dir_all("_test").unwrap();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open("_test/md_test_5.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
        assert_eq!(pico.put_metadata(10, b"Martindal").unwrap(), 0);
        let mut data = [0u8; 20];
        // The result here might be 10, or it might be zero.  It depends on whether or
        // not the metadata has been padded out.  We don't actually care, do we don't
        // check.
        pico.get_metadata(0, &mut data).unwrap();
        assert_eq!(&data, b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
        remove_file("_test/md_test_5.pico").unwrap();
    }

    #[test]
    fn data_test_1() {
        create_dir_all("_test").unwrap();
        {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open("_test/data_test_1.pico")
                .unwrap();
            let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 10).unwrap();
            let mut indata = b"Martindale".clone();
            pico.put(10, &mut indata).unwrap();
            pico.flush().unwrap();
        }
        {
            let file = OpenOptions::new()
                .create(false)
                .write(true)
                .read(true)
                .open("_test/data_test_1.pico")
                .unwrap();
            let mut pico = Pico::open(file).unwrap();
            let mut data = [0u8; 20];
            assert_eq!(pico.get(0, &mut data).unwrap(), 20);
            assert_eq!(&data[10..], b"Martindale");
        }
        remove_file("_test/data_test_1.pico").unwrap();
    }
}
