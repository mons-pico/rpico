extern crate pico;

use std::fs::OpenOptions;
use pico::{Pico, HeaderFormat};

fn main() {
    {
        let mut out = std::io::stdout();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("testify.pico")
            .unwrap();
        let mut pico = Pico::new(file, vec![0x55, 0x21, 0xe4, 0x9a], 0).unwrap();
        pico.flush().unwrap();
        pico.dump_header(&mut out, HeaderFormat::DICT);
        pico.dump_header(&mut out, HeaderFormat::JSON);
        pico.dump_header(&mut out, HeaderFormat::YAML);
        pico.dump_header(&mut out, HeaderFormat::XML);
    }
    // {
    //     let file = File::open("testify.pico").unwrap();
    //     let mut pico = Pico::open(file).unwrap();
    // }
}
