//! Exporting Pico file header information.

use std::str::FromStr;
use std::result;

/// Different formats for writing out the header.
#[derive(Debug)]
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

impl FromStr for HeaderFormat {
    type Err = String;
    fn from_str(name: &str) -> result::Result<HeaderFormat, Self::Err> {
        let ciname = name.to_uppercase();
        match ciname.as_str() {
            "DICT" => Ok(HeaderFormat::DICT),
            "JSON" => Ok(HeaderFormat::JSON),
            "YAML" => Ok(HeaderFormat::YAML),
            "XML" => Ok(HeaderFormat::XML),
            _ => Err(
                format!("Unknown header format: {}", ciname).to_string()
            )
        }
    }
}
