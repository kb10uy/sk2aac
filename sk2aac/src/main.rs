mod data;
mod emitter;

use crate::data::AnimationDescriptor;

use std::{
    env::args,
    fs::{read_to_string, File},
    io::BufWriter,
};

use anyhow::{bail, Result};
use emitter::emit_descriptor;
use serde_json::from_str as json_from_str;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() <= 2 {
        bail!("Usage: sk2aac <descriptor JSON> <output cs>");
    }
    let descriptor: AnimationDescriptor = json_from_str(&read_to_string(&args[1])?)?;

    let mut output_file = BufWriter::new(File::create(&args[2])?);
    let class_name = emit_descriptor(&mut output_file, &descriptor)?;
    println!("You should rename the file to {class_name}.cs");

    Ok(())
}
