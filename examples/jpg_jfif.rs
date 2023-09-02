#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read("examples/samples/sample_jfif.jpeg")?;
    let jpeg = quickexif::jpeg::JPEG::new(&data)?;
    println!("{:x?}", jpeg);
    Ok(())
}
