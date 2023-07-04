#![allow(dead_code)]
#![allow(unused_imports)]

use std::{collections::HashMap, fs::File, io::BufReader};
use quickexif::log_helper::*;

mod cr2_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x010f make
            0x0110 model
            0x0112 orientation
            0x0111 thumbnail
            0x0117 thumbnail_length
        }
        0 -> 0x8769 -> 0 {
            0xa002 width
            0xa003 height
        }
        0 -> 0x8769 -> 0 -> 0x927c -> 0 {
            0x4001 colordata
        }
        3 {
            0x0111 strip
            0x0117 strip_count
            0xc5e0 cfa_pattern
        }
    );
}

fn main() -> LogResult<()> {
    let sample = "examples/samples/sample0.CR2";
    let reader = BufReader::new(q!(File::open(sample)));

    let result = q!(quickexif::parse_exif(reader, cr2_tags::PATH_LST, None));

    println!("{:?}", result.get(cr2_tags::make).and_then(|x| x.str()));
    println!("{:?}", result.get(cr2_tags::model).and_then(|x| x.str()));
    println!("{:?}", result.get(cr2_tags::orientation).map(|x| x.u16()));
    println!("{:?}", result.get(cr2_tags::thumbnail).map(|x| x.u32()));
    println!("{:?}", result.get(cr2_tags::thumbnail_length).map(|x| x.u32()));
    println!("{:x?}", result.get(cr2_tags::colordata).map(|x| x.raw()));
    println!("{:x?}", result.get(cr2_tags::strip).map(|x| x.u32()));
    println!("{:x?}", result.get(cr2_tags::strip_count).map(|x| x.u32()));
    println!("{:x?}", result.get(cr2_tags::cfa_pattern).map(|x| x.raw()));

    Ok(())
}
