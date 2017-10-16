//! Command line executable.
extern crate pico;
extern crate clap;

use std::str::FromStr;
use std::path::Path;
use std::io::stdout;
use pico::{HeaderFormat, major, minor};
use clap::{Arg, App};
use pico::file;

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
    let extension = match app_matches.value_of("extension") {
        None => {
            match op {
                Operation::Decode => ".raw",
                _ => ".pico",
            }
        },
        Some(ext) => ext,
    };
    let suffix = app_matches.value_of("suffix").unwrap();

    // Perform the operation for each specified file.
    for file in filelist {
        let filepath = Path::new(&file);
        let basename = filepath.file_stem().unwrap().to_string_lossy().into_owned();
        let oldname: String = filepath.to_string_lossy().into_owned();
        match op {
            Operation::Header => {
                println!("Pico Header as {:?} for: {:?}", header_format, filepath);
                file::dump_header(&oldname, stdout(), &header_format);
            },
            Operation::Encode => {
                let key = pico::gen_random_key(16);
                let newname = basename + suffix + extension;
                println!("Encoding {:?} -> {:?}", oldname, newname);
                file::encode(&oldname, &newname, key, vec![], 0);
            },
            Operation::Decode => {
                let newname = basename + suffix + extension;
                println!("Decoding {:?} -> {:?}", oldname, newname);
                file::decode(&oldname, &newname);
            },
        }
    }
}
