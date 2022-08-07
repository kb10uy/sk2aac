mod code_writer;
mod data;
mod data2;
mod data_raw;
mod emitter;

use crate::{data::AnimationDescriptor, data2::Descriptor};

use std::{
    env::args,
    fs::{read_to_string, File},
    io::BufWriter,
};

use anyhow::{bail, Result};
use emitter::emit_descriptor;
use serde_json::from_str as json_from_str;
use toml::from_str as toml_from_str;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() <= 2 {
        bail!("Usage: sk2aac <descriptor TOML> <output cs>");
    }
    let descriptor: Descriptor = toml_from_str(&read_to_string(&args[1])?)?;
    println!("{descriptor:?}");

    // let mut output_file = BufWriter::new(File::create(&args[2])?);
    // let class_name = emit_descriptor(&mut output_file, &descriptor)?;
    // println!("You should rename the file to {class_name}.cs");

    Ok(())
}
