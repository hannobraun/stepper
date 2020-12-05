use std::{
    env,
    error::Error,
    fs::{create_dir_all, remove_dir_all, File},
    io::prelude::*,
    path::PathBuf,
};

use serde_derive::Serialize;
use serde_json::Value;
use tinytemplate::{format_unescaped, TinyTemplate};

mod cargo_task_util;

mod config;
use config::{load_cargo_toml, load_drivers_toml, Driver};

fn main() -> Result<(), Box<dyn Error>> {
    // `root`      - executing directory; assumed to be Step/Dir root
    // `drivers`   - driver facade crate directory (output)
    // `templates` - template directory (input)
    let root = env::current_dir()?;
    let drivers = root.join("drivers");
    let templates = root.join("templates").join("driver");

    // Set up the template engine. By default we will *not* escape any input.
    let mut tt = TinyTemplate::new();
    tt.set_default_formatter(&format_unescaped);
    tt.add_formatter("upper", format_upper);

    // Load the 'Cargo.toml', 'src/lib.rs', and 'README.md' templates and
    // register them with the template engine.
    let cargo_toml = load_template(&templates.join("Cargo.toml.tmpl"))?;
    tt.add_template("cargo_toml", cargo_toml.as_str())?;

    let lib_rs = load_template(&templates.join("src").join("lib.rs.tmpl"))?;
    tt.add_template("lib_rs", lib_rs.as_str())?;

    let readme_md = load_template(&templates.join("README.md.tmpl"))?;
    tt.add_template("readme_md", readme_md.as_str())?;

    // Load the project's version and authors from the root Step/Dir
    // `Cargo.toml` file.
    let manifest = load_cargo_toml(&root)?;
    let version = manifest.package.version;
    let authors = manifest.package.authors;

    // Load the configuration and generate each configured driver.
    let config = load_drivers_toml(&root)?;
    for driver in config.drivers {
        ct_info!("generating '{}' driver...", driver.name);

        // Build the rendering Context struct to pass along to the template
        // engine.
        let ctx = &Context::new(driver, &version, &authors);

        // If the driver crate path already exists, just delete it.
        let driver_path = drivers.join(&ctx.name);
        if driver_path.exists() {
            remove_dir_all(&driver_path)?;
        }

        // Create the driver directory, as well as its 'src' subdirectory.
        create_dir_all(&driver_path.join("src"))?;

        // Render each template using the current `Context` instance.
        let cargo_toml_output = tt.render("cargo_toml", ctx)?;
        let lib_rs_output = tt.render("lib_rs", ctx)?;
        let readme_md_output = tt.render("readme_md", ctx)?;

        // Create each output file and write out their contents.
        File::create(&driver_path.join("Cargo.toml"))?
            .write_all(cargo_toml_output.as_ref())?;

        File::create(&driver_path.join("src").join("lib.rs"))?
            .write_all(lib_rs_output.as_ref())?;

        File::create(&driver_path.join("README.md"))?
            .write_all(readme_md_output.as_ref())?;
    }

    Ok(())
}

/// The template rendering context. The values contained in this struct are used
/// to perform substitutions within the templates.
#[derive(Serialize)]
struct Context {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub product_url: String,
    pub pololu_url: String,
}

impl Context {
    pub fn new(
        driver: Driver,
        version: &String,
        authors: &Vec<String>,
    ) -> Self {
        Self {
            name: driver.name,
            version: version.to_owned(),
            authors: authors.to_owned(),
            product_url: driver.product_url,
            pololu_url: driver.pololu_url,
        }
    }
}

fn format_upper(
    value: &Value,
    output: &mut String,
) -> Result<(), tinytemplate::error::Error> {
    let mut s = String::new();
    format_unescaped(value, &mut s)?;

    output.push_str(&s.to_uppercase());

    Ok(())
}

fn load_template(path: &PathBuf) -> std::io::Result<String> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    Ok(contents)
}
