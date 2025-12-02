use std::{path::Path, process::Command};

pub fn write_svg(input_path: &Path) {
    let output_path = input_path.with_extension("svg");

    let status = Command::new("dot")
        .arg("-Tsvg")
        .arg(input_path)
        .arg("-o")
        .arg(&output_path)
        .status()
        .expect("failed to execute process");
    if status.success() {
        println!("{:?} created.", output_path);
    } else {
        eprintln!("dot failed with: {}", status);
    }
}
