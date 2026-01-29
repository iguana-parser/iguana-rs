use std::{io, path::Path, process::Command};

pub fn write_svg(input_path: &Path) -> io::Result<()> {
    let output_path = input_path.with_extension("svg");

    Command::new("dot")
        .arg("-Tsvg")
        .arg(input_path)
        .arg("-o")
        .arg(&output_path)
        .status()?;

    Ok(())
}
