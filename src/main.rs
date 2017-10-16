//! Command line executable.
extern crate pico;
extern crate clap;
extern crate hex;

use std::str::FromStr;
use std::path::Path;
use std::io::stdout;
use pico::{HeaderFormat, major, minor};
use clap::{Arg, App};
use pico::file;
use hex::FromHex;

/// Executable description.
static DESCRIPTION: &str =
"Encode a file as Pico, decode a Pico-encoded file, or dump the header \
from a Pico-encoded file.";

static LONG_DESCRIPTION: &str =
"Input files are encoded by default.  If encoding, a .pico extension \
is added to the file.  If decoding, then the input must be Pico-encoded \
files, and a .raw extension is added by default.  If dumping the header, \
the input files must be Pico-encoded files, and the header is sent to \
standard output in the specified format.

The extension used can be overridden by --extension, which should include \
the dot.  Any provided suffix (by default there is none) is added to the \
file's base name.

The header kinds can be JSON, YAML, DICT (Python), or XML.

Keys must be specified as a list of hexadecimal digits (no spaces).  If \
no key is specified for encoding, a random key is generated.";

/// Entry point when run from the command line.
fn main() {
    // Add some information to the end of the help.
    let after = format!(
        "{}\n\nPico Encoding Version: {}.{}\nSee: {}",
        LONG_DESCRIPTION, major(), minor(), env!("CARGO_PKG_HOMEPAGE")
    );

    // Parse command line arguments.
    let app_matches = App::new("Pico Rust Library")
        .version(env!("CARGO_PKG_VERSION"))
        .author("The Mons Pico Project")
        .about(DESCRIPTION)
        .after_help(after.as_str())
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Increase verbosity.")
            .takes_value(false))
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Enable debugging.")
            .takes_value(false))
        .arg(Arg::with_name("decode")
            .conflicts_with("encode")
            .conflicts_with("header")
            .short("d")
            .long("decode")
            .help("Decode files.")
            .takes_value(false))
        .arg(Arg::with_name("encode")
            .conflicts_with("decode")
            .conflicts_with("header")
            .short("e")
            .long("encode")
            .help("Encode files.")
            .takes_value(false))
        .arg(Arg::with_name("extension")
            .long("extension")
            .help("Set output file extension.")
            .takes_value(true))
        .arg(Arg::with_name("header")
            .conflicts_with("encode")
            .conflicts_with("decode")
            .possible_values(&["DICT", "JSON", "YAML", "XML"])
            .short("H")
            .long("header")
            .value_name("format")
            .help("Dump header information.")
            .takes_value(true))
        .arg(Arg::with_name("suffix")
            .short("s")
            .long("suffix")
            .default_value("")
            .help("Suffix to add to output files.")
            .takes_value(true))
        .arg(Arg::with_name("key")
            .short("k")
            .long("key")
            .help("Specify key for encoding.")
            .takes_value(true))
        .arg(Arg::with_name("files")
            .help("File names to process.")
            .multiple(true)
            .required(true)
            .takes_value(true))
        .get_matches();

    // Figure out correct operation.  This unwrap should not fail since
    // the files are required.
    let filelist = app_matches.values_of("files").unwrap();
    enum Operation {
        Header, Encode, Decode,
    };
    let mut op = Operation::Encode;
    if app_matches.is_present("header") { op = Operation::Header; }
    if app_matches.is_present("decode") { op = Operation::Decode; }
    let header_format = match app_matches.value_of("header") {
        None => HeaderFormat::DICT,
        // This unwrap should not fail, since the format names are checked
        // when parsing the command line.
        Some(name) => HeaderFormat::from_str(name).unwrap(),
    };
    let extension = match app_matches.value_of("extension") {
        None => {
            match op {
                Operation::Decode => ".raw",
                _ => ".pico",
            }
        },
        Some(ext) => ext,
    };
    // This unwrap should never fail since suffix has a default value.
    let suffix = app_matches.value_of("suffix").unwrap();

    // Perform the operation for each specified file.
    for file in filelist {
        // Check the file.
        let filepath = Path::new(&file);
        if filepath.is_dir() {
            eprintln!("ERROR: Argument {:?} is a folder.", file);
            return;
        }
        if !filepath.exists() {
            eprintln!("ERROR: Argument {:?} is not found.", file);
            return;
        }
        let basename = match filepath.file_stem() {
            None => {
                eprintln!("ERROR: Argument {:?} is not a file.", file);
                return;
            },
            Some(value) => value,
        }.to_string_lossy().into_owned();
        let oldname: String = filepath.to_string_lossy().into_owned();

        // Perform the correct operation.
        match op {
            Operation::Header => {
                println!("Pico Header as {:?} for: {:?}", header_format, filepath);
                match file::dump_header(&oldname, stdout(), &header_format) {
                    Ok(()) => (),
                    Err(err) => eprintln!("ERROR: {}", err),
                };
            },

            Operation::Encode => {
                // See if the user specified a key; if not, generate one.
                let key = match app_matches.value_of("key") {
                    None => pico::gen_random_key(16),
                    Some(hex) => {
                        let hex = hex.to_uppercase().into_bytes();
                        let hexlen = hex.len();
                        if hexlen % 2 != 0 {
                            // I think this is more helpful than the default given
                            // by the hex package.
                            eprintln!("ERROR: Key must be an even number of hex digits.");
                            return;
                        }
                        if hexlen == 0 {
                            // The hex package permits an empth string, so we have
                            // to trap this here.
                            eprintln!("ERROR: Key cannot be empty.");
                            return;
                        }
                        match Vec::<u8>::from_hex(hex) {
                            Ok(value) => value,
                            Err(err) => {
                                eprintln!("ERROR: {}", err);
                                return;
                            }
                        }
                    }
                };
                let newname = basename + suffix + extension;
                println!("Encoding {:?} -> {:?}", oldname, newname);
                match file::encode(&oldname, &newname, key, vec![], 0) {
                    Ok(()) => (),
                    Err(err) => eprintln!("ERROR: {}", err),
                };
            },

            Operation::Decode => {
                let newname = basename + suffix + extension;
                println!("Decoding {:?} -> {:?}", oldname, newname);
                match file::decode(&oldname, &newname) {
                    Ok(()) => (),
                    Err(err) => eprintln!("ERROR: {}", err),
                };
            },
        }
    }
}
