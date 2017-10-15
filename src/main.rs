//! Command line executable.
extern crate pico;
extern crate clap;

use pico::{HeaderFormat, major, minor};
use clap::{Arg, App};
use std::str::FromStr;

/// Executable description.
static DESCRIPTION: &str =
"Encode a file as Pico, decode a Pico-encoded file, or dump the header \
from a Pico-encoded file.";

static LONG_DESCRIPTION: &str =
"Input files are encoded by default.  If encoding, a .pico extension \
is added to the file.  If decoding, then the input must be Pico-encoded \
files, and a .raw extension is added by default.  If dumping the header, \
the input files must be Pico-encoded files, and the header is dumped as \
JSON to standard output.

The extension used can be overridden by --extension, which should include \
the dot.  Any provided suffix (by default there is none) is added to the \
file's base name.

The header kinds can be json, yaml, dict (python), or xml.

Keys must be specified as a list of hexadecimal digits (no spaces).";

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
            .short("H")
            .long("header")
            .value_name("format")
            .help("Dump header information.")
            .takes_value(true))
        .arg(Arg::with_name("suffix")
            .short("s")
            .long("suffix")
            .help("Suffix to add to output files.")
            .takes_value(true))
        .arg(Arg::with_name("files")
            .help("File names to process.")
            .multiple(true)
            .required(true)
            .takes_value(true))
        .get_matches();

    // Figure out correct operation.
    let filelist = app_matches.values_of("files").unwrap();
    enum Operation {
        Header, Encode, Decode,
    };
    let mut op = Operation::Encode;
    if app_matches.is_present("header") { op = Operation::Header; }
    if app_matches.is_present("decode") { op = Operation::Decode; }
    let header_format = match app_matches.value_of("header") {
        None => HeaderFormat::DICT,
        Some(name) => HeaderFormat::from_str(name).unwrap(),
    };

    // Perform the operation for each specified file.
    for file in filelist {
        match op {
            Operation::Header => {
                println!("Pico Header as {:?} for: {}", header_format, file);
            },
            Operation::Encode => {
                println!("Encoding {}...", file);
            },
            Operation::Decode => {
                println!("Decoding {}...", file);
            },
        }
    }
}
