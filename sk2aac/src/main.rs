mod codegen;
mod descriptor;

use crate::{
    codegen::write_descriptor_code,
    descriptor::{validate_descriptor, Descriptor},
};

use std::{
    env::args,
    fs::{read_to_string, File},
    io::BufWriter,
};

use anyhow::{bail, Result};
use toml::from_str as toml_from_str;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() <= 2 {
        bail!("Usage: sk2aac <descriptor TOML> <output cs>");
    }

    let descriptor: Descriptor = toml_from_str(&read_to_string(&args[1])?)?;
    validate_descriptor(&descriptor)?;

    let mut output_file = BufWriter::new(File::create(&args[2])?);
    let class_name = write_descriptor_code(&mut output_file, descriptor)?;
    println!("You should rename the file to {class_name}.cs");

    Ok(())
}
