use std::{
    fs::File,
    io::{prelude::*, Result},
    path::PathBuf,
};

use serde_derive::Deserialize;

/// Values required to populate the 'Cargo.toml', 'lib.rs', and 'README.md'
/// templates for a given driver implementation.
#[derive(Debug, Deserialize)]
pub struct Driver {
    pub name: String,
    pub product_url: String,
    pub pololu_url: String,
}

/// The 'drivers.toml' file format. Consists of one or more drivers.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub drivers: Vec<Driver>,
}

pub fn load_drivers_toml(root: &PathBuf) -> Result<Config> {
    // It is assumed that 'drivers.toml' exists in the project's root.
    let path = root.join("drivers.toml");
    assert!(path.exists());

    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    let config: Config = toml::from_str(contents.as_str())?;

    Ok(config)
}

/// Cargo manifest package values. The driver facade crates will have their
/// `version` and `authors` fields kept in sync with the `step_dir` crate.
#[derive(Debug, Deserialize)]
pub struct Package {
    pub version: String,
    pub authors: Vec<String>,
}

/// The 'Cargo.toml' file format. We're only concerned with the '\[package\]'
/// section at this time.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub package: Package,
}

pub fn load_cargo_toml(root: &PathBuf) -> Result<Manifest> {
    // It is assumed that 'Cargo.toml' exists in the project's root.
    let path = root.join("Cargo.toml");
    assert!(path.exists());

    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    let manifest: Manifest = toml::from_str(contents.as_str())?;

    Ok(manifest)
}
