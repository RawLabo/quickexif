#![allow(dead_code)]
#![allow(unused_imports)]

use std::{collections::HashMap, fs::File, io::BufReader};
use quickexif::report::*;

mod jpg_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x010f make
            0x0110 model
        }
        0 -> 0x8769 -> 0 {
            0x8827 iso
            0x829a exposure_time
            0x829d f_number
            0x9004 create_date
            0x920a focal_length
            0xa002 width
            0xa003 height
        }
        1 {
            0x0201 thumb_addr
            0x0202 thumb_len
        }
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = "examples/samples/sample0.JPG";
    let reader = BufReader::new(File::open(sample)?);

    let result = quickexif::parse_exif(reader, jpg_tags::PATH_LST, None)?;

    println!("{:?}", result.get(jpg_tags::make).and_then(|x| x.str()));
    println!("{:?}", result.get(jpg_tags::model).and_then(|x| x.str()));

    println!("{:?}", result.get(jpg_tags::iso).map(|x| x.u16()));
    println!("{:?}", result.get(jpg_tags::exposure_time).and_then(|x| x.r64s()));
    println!("{:?}", result.get(jpg_tags::f_number).and_then(|x| x.r64s()));
    println!("{:?}", result.get(jpg_tags::create_date).and_then(|x| x.str()));
    println!("{:?}", result.get(jpg_tags::focal_length).and_then(|x| x.r64s()));
    println!("{:?}", result.get(jpg_tags::width).map(|x| x.u32()));
    println!("{:?}", result.get(jpg_tags::height).map(|x| x.u32()));
    println!("{:?}", result.get(jpg_tags::thumb_addr).map(|x| x.u32()));
    println!("{:?}", result.get(jpg_tags::thumb_len).map(|x| x.u32()));

    Ok(())
}
