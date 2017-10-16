//! File operations for Pico encoding and decoding.

use pico::Pico;
use header::HeaderFormat;
use constants::CHUNK_SIZE;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use errors::{Result, PicoError};

pub fn encode(
    from: &String, 
    to: &String, 
    key: Vec<u8>, 
    metadata: Vec<u8>, 
    reserve: u32) -> Result<()> {
    // Open the file to read.
    let mut source = OpenOptions::new()
        .create(false)
        .read(true)
        .open(from)
        .map_err(|err| {
            PicoError::FileNotFound(2001, from.clone(), err)
        })?;

    // Open the file to write.
    let target = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(to)
        .map_err(|err| {
            PicoError::FileExists(2002, to.clone(), err)
        })?;

    // Create the Pico structure.
    let mut pico = Pico::new(target, key, reserve)?;

    // Write the metadata.
    pico.put_metadata(0, &metadata)?;

    // Now read chunks from the input file and write them encoded
    // into the output file.
    let mut position: usize = 0;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    loop {
        // Read a chunk from the input file.
        let count = source.read(&mut buffer)
            .map_err(|err| { PicoError::ReadFailed(2003, err) })?;
        if count == 0 { break; }

        // Encode and write the chunk to the output file.
        pico.put(position, &mut buffer[0..count])?;
        position += count;
    }

    // Done encoding.  Flush the Pico file and then let the
    // files get dropped, which closes them.
    pico.flush()?;
    Ok(())
}

pub fn decode(
    from: &String, 
    to: &String) -> Result<()> {
    // Open the file to read.
    let source = OpenOptions::new()
        .create(false)
        .read(true)
        .write(true)
        .open(from)
        .map_err(|err| {
            PicoError::FileNotFound(2010, from.clone(), err)
        })?;

    // Open the file to write.
    let mut target = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(to)
        .map_err(|err| {
            PicoError::FileExists(2011, to.clone(), err)
        })?;

    // Create the Pico structure.
    let mut pico = Pico::open(source)?;

    // Now read chunks from the input file and write them decoded
    // into the output file.
    let mut position: usize = 0;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    loop {
        // Read a chunk from the input file.
        let count = pico.get(position, &mut buffer)?;
        if count == 0 { break; }

        // Encode and write the chunk to the output file.
        target.write(&buffer[0..count])
            .map_err(|err| { PicoError::WriteFailed(2012, err) })?;
        position += count;
    }

    // Done encoding.  Flush the Pico file and then let the
    // files get dropped, which closes them.
    pico.flush()?;
    Ok(())
}

pub fn dump_header<W: Write>(
    from: &String, 
    mut to: W,
    format: &HeaderFormat) -> Result<()> {
    // Open the file to read.
    let source = OpenOptions::new()
        .create(false)
        .read(true)
        .write(true)
        .open(from)
        .map_err(|err| {
            PicoError::FileNotFound(2020, from.clone(), err)
        })?;

    // Create the Pico structure.
    let pico = Pico::open(source)?;

    // Write the header.
    pico.dump_header(&mut to, format);
    Ok(())
}