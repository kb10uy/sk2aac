mod codegen;
mod descriptor;

use crate::{codegen::AacCodeGenerator, descriptor::Descriptor};

use std::{
    env::args,
    fs::{read_to_string, File},
    io::BufWriter,
};

use anyhow::{bail, Result};
// use emitter::emit_descriptor;
// use serde_json::from_str as json_from_str;
use toml::from_str as toml_from_str;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() <= 2 {
        bail!("Usage: sk2aac <descriptor TOML> <output cs>");
    }

    let descriptor: Descriptor = toml_from_str(&read_to_string(&args[1])?)?;
    let mut output_file = BufWriter::new(File::create(&args[2])?);
    let mut acg = AacCodeGenerator::new(&mut output_file, &descriptor.name)?;
    acg.emit_code(&descriptor)?;
    println!("You should rename the file to {}.cs", acg.class_name());

    Ok(())
}
