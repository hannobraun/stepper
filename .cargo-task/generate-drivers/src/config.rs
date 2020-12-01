use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, Result};
use std::path::PathBuf;

use serde_derive::Deserialize;

// Values required to populate the 'Cargo.toml', 'lib.rs', and 'README.md'
// templates for a given driver implementation.
#[derive(Debug, Deserialize)]
pub struct Driver {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub product_url: String,
    pub pololu_url: String,
}

// The 'drivers.toml' file format.
#[derive(Debug, Deserialize)]
pub struct Config {
    // Key:   driver name
    // Value: driver configuration
    pub drivers: HashMap<String, Driver>,
}

pub fn load_drivers_toml(root: &PathBuf) -> Result<Config> {
    // It is assumed that 'drivers.toml' exists in the project's root.
    let path = root.join("drivers.toml");
    assert!(path.exists());

    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(contents.as_str())
        .expect("unable to parse 'drivers.toml'");

    Ok(config)
}
