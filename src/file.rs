//! File operations and core data structure for the Pico library.

use std::io::{Read, Write, Seek, SeekFrom};
use constants::*;
use crypt::crypt;
use intbytes::{ByteDump, dump_vec};
use errors::{PicoError, Result};
use md5;

/// Different formats for writing out the header.
pub enum HeaderFormat {
    /// Use Python dict format.  This should work in both Python 2.7+
    /// and in Python 3+.
    ///
    /// # Example
    /// ```python
    /// {
    ///     "magic" : [ 0x91, 0xC0 ],
    ///     "major" : 1,
    ///     "minor" : 0,
    ///     "offset" : 42,
    ///     "hash" : [ 0xD4, 0x1D, 0x8C, 0xD9, 0x8F, 0x00, 0xB2, 0x04,
    ///                0xE9, 0x80, 0x09, 0x98, 0xEC, 0xF8, 0x42, 0x7E ],
    ///     "key_length" : 4,
    ///     "key" : [ 0x55, 0x21, 0xE4, 0x9A ],
    ///     "md_length" : 10,
    /// }
    /// ```
    DICT,
    /// Use JSON format.  See http://www.json.org.
    ///
    /// # Example
    /// ```json
    /// {
    ///     "magic" : [ 145, 192 ],
    ///     "major" : 1,
    ///     "minor" : 0,
    ///     "offset" : 42,
    ///     "hash" : [ 212, 29, 140, 217, 143, 0, 178, 4,
    ///                233, 128, 9, 152, 236, 248, 66, 126 ],
    ///     "key_length" : 4,
    ///     "key" : [ 85, 33, 228, 154 ],
    ///     "md_length" : 10,
    /// }
    /// ```
    JSON,
    /// Use YAML format.  Currently targeting the 1.2 version of the YAML
    /// standard.  See http://yaml.org.
    ///
    /// # Example
    /// ```yaml
    /// magic: [ 145, 192 ]
    /// major: 1
    /// minor: 0
    /// offset: 42
    /// hash: [ 212, 29, 140, 217, 143, 0, 178, 4,
    ///         233, 128, 9, 152, 236, 248, 66, 126 ]
    /// key_length: 4
    /// key: [ 85, 33, 228, 154 ]
    /// md_length: 10
    /// ```
    YAML,
    /// Use XML format.  All data is part of a single element, with data
    /// values provided by attributes.
    ///
    /// # Example
    /// ```xml
    /// <pico magic='0x91C0' major='1' minor='0' offset='42'
    ///       hash='D41D8CD98F00B204E9800998ECF8427E key='5521E49A
    ///       md_length='10' />
    /// ```
    XML,
}

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
            ((buf[1] as u16) << 8) + (buf[0] as u16)
        }
        fn tou32(buf: [u8; 4]) -> u32 {
            ((buf[1] as u32) << 24) + ((buf[0] as u32) << 16) + ((buf[1] as u32) << 8) +
                (buf[0] as u32)
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
        let mut key = Vec::<u8>::with_capacity(keylen as usize);
        file.read(key.as_mut_slice()).map_err(|err| {
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
        // Compute the true offset to the data.
        let true_offset = start as usize + self.md_start;

        // Compute the maximum number of bytes that can be read from the
        // metadata, and then figure out how many bytes we actually need to
        // read.
        let mut max = self.get_offset() as usize - true_offset;
        if max < buffer.len() {
            max = buffer.len();
        }
        if max <= 0 {
            return Ok(0);
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
        // Compute the true offset to the data.
        let true_offset = start as usize + self.md_start;

        // Compute the maximum number of bytes that can be written to the
        // metadata, and then figure out how many bytes we actually need to
        // write.
        let mut max = self.get_offset() as usize - true_offset;
        if max < buffer.len() {
            max = buffer.len();
        }
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