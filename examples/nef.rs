#![allow(dead_code)]
#![allow(unused_imports)]

use std::{collections::HashMap, fs::File, io::BufReader};

mod nikon_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x0112 orientation
        }
        0 -> 0x8769 -> 0 {
            0xa302 cfa_pattern
        }
        0 -> 0x8769 -> 0 -> 0x927c -> 0 {
            0x000c white_balance
            0x003d black_level
            0x0045 crop_area
            0x008c contrast_curve
            0x0096 linear_table
        }
        0 -> 0x014a -> 0 {
            0x0201 thumbnail
            0x0202 thumbnail_len
        }
        0 -> 0x014a -> 100 {
            0x0100 width
            0x0101 height
            0x0102 bps
            0x0103 compression
            0x0111 strip
            0x0117 strip_len
        }
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = "examples/samples/sample0.NEF";
    let reader = BufReader::new(File::open(sample)?);

    let (result, _) = quickexif::parse_exif(reader, nikon_tags::PATH_LST, None)?;

    println!("{:?}", result.get(nikon_tags::orientation).map(|x| x.u16()));
    println!("{:?}", result.get(nikon_tags::thumbnail).map(|x| x.u32()));
    println!("{:?}", result.get(nikon_tags::thumbnail_len).map(|x| x.u32()));
    println!("{:?}", result.get(nikon_tags::cfa_pattern).map(|x| x.raw()));
    println!("{:?}", result.get(nikon_tags::white_balance).and_then(|x| x.r64s()));
    println!("{:?}", result.get(nikon_tags::black_level).and_then(|x| x.u16s()));
    println!("{:?}", result.get(nikon_tags::crop_area).and_then(|x| x.u16s()));
    println!("{:x?}", result.get(nikon_tags::contrast_curve).map(|x| x.raw()));
    println!("{:x?}", result.get(nikon_tags::linear_table).map(|x| x.raw()));
    println!("{:?}", result.get(nikon_tags::width).map(|x| x.u32()));
    println!("{:?}", result.get(nikon_tags::height).map(|x| x.u32()));
    println!("{:?}", result.get(nikon_tags::bps).map(|x| x.u16()));
    println!("{:?}", result.get(nikon_tags::compression).map(|x| x.u16()));
    println!("{:?}", result.get(nikon_tags::strip).map(|x| x.u32()));
    println!("{:?}", result.get(nikon_tags::strip_len).map(|x| x.u32()));

    Ok(())
}
