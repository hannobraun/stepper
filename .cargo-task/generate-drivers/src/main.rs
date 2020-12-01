use std::{
    env,
    fs::{create_dir_all, remove_dir_all, File},
    io::prelude::*,
    path::PathBuf,
};

use serde_derive::Serialize;
use serde_json::Value;
use tinytemplate::{error::Error, format_unescaped, TinyTemplate};

mod cargo_task_util;

mod config;
use config::{load_drivers_toml, Driver};

fn main() -> std::io::Result<()> {
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
    tt.add_template("cargo_toml", cargo_toml.as_str())
        .expect("unable to add template");

    let lib_rs = load_template(&templates.join("src").join("lib.rs.tmpl"))?;
    tt.add_template("lib_rs", lib_rs.as_str())
        .expect("unable to add template");

    let readme_md = load_template(&templates.join("README.md.tmpl"))?;
    tt.add_template("readme_md", readme_md.as_str())
        .expect("unable to add template");

    // Load the configuration and generate each configured driver.
    let config = load_drivers_toml(&root)?;
    for driver in config.drivers.values() {
        ct_info!("generating '{}' driver...", driver.name);

        // This is annoying and sort of hacky. This converts the `Vec` of
        // authors to a formatted string, and copies the remaining values over
        // verbatim.
        let ctx = &Context::from(driver);

        // If the driver crate path already exists, just delete it.
        let driver_path = drivers.join(ctx.name.clone());
        if driver_path.exists() {
            remove_dir_all(&driver_path)?;
        }

        // Create the driver directory, as well as its 'src' subdirectory.
        create_dir_all(&driver_path.join("src"))?;

        // Render each template using the current `Context` instance.
        let cargo_toml_output = tt
            .render("cargo_toml", ctx)
            .expect("unable to render template");

        let lib_rs_output =
            tt.render("lib_rs", ctx).expect("unable to render template");

        let readme_md_output = tt
            .render("readme_md", ctx)
            .expect("unable to render template");

        // Create each output file and write out their contents.
        File::create(&driver_path.join("Cargo.toml"))
            .expect("error writing file")
            .write_all(cargo_toml_output.as_ref())?;

        File::create(&driver_path.join("src").join("lib.rs"))
            .expect("error writing file")
            .write_all(lib_rs_output.as_ref())?;

        File::create(&driver_path.join("README.md"))
            .expect("error writing file")
            .write_all(readme_md_output.as_ref())?;
    }

    Ok(())
}

#[derive(Serialize)]
struct Context {
    pub name: String,
    pub version: String,
    pub authors: String,
    pub product_url: String,
    pub pololu_url: String,
}

impl From<&Driver> for Context {
    fn from(driver: &Driver) -> Context {
        Context {
            name: driver.name.clone(),
            version: driver.version.clone(),
            authors: format_authors(driver.authors.clone()),
            product_url: driver.product_url.clone(),
            pololu_url: driver.pololu_url.clone(),
        }
    }
}

fn format_upper(value: &Value, output: &mut String) -> Result<(), Error> {
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

fn format_authors(authors: Vec<String>) -> String {
    assert!(authors.len() != 0);

    // If only a single author has been specified, just return them.
    // Remember to wrap the String in quotes!
    if authors.len() == 1 {
        return format!("\"{}\"", authors[0]);
    }

    // If multiple authors have been specified, add each to a new (indented)
    // line with trailing commas.
    let mut output = String::from("\n");
    for author in authors {
        let a = format!("    \"{}\",\n", author);
        output.push_str(a.as_str());
    }

    output
}
