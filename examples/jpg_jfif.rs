#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("examples/samples/sample_jfif.jpeg")?;
    let jpeg = quickexif::jpeg::JPEG::new(f)?;
    Ok(())
}