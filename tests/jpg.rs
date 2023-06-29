#![allow(dead_code)]
#![allow(unused_imports)]

use log::info;
use std::{collections::HashMap, fs::File};
use quickexif::log_helper::*;

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

#[test]
fn parse_jpg() -> LogResult<()> {
    env_logger::init();
    let sample = "tests/samples/sample0.JPG";
    let f = q!(File::open(sample));

    let result = q!(quickexif::parse_exif(f, jpg_tags::PATH_LST, None));

    info!("{:?}", result.get(jpg_tags::make).and_then(|x| x.str()));
    info!("{:?}", result.get(jpg_tags::model).and_then(|x| x.str()));

    info!("{:?}", result.get(jpg_tags::iso).map(|x| x.u16()));
    info!("{:?}", result.get(jpg_tags::exposure_time).and_then(|x| x.r64s()));
    info!("{:?}", result.get(jpg_tags::f_number).and_then(|x| x.r64s()));
    info!("{:?}", result.get(jpg_tags::create_date).and_then(|x| x.str()));
    info!("{:?}", result.get(jpg_tags::focal_length).and_then(|x| x.r64s()));
    info!("{:?}", result.get(jpg_tags::width).map(|x| x.u32()));
    info!("{:?}", result.get(jpg_tags::height).map(|x| x.u32()));
    info!("{:?}", result.get(jpg_tags::thumb_addr).map(|x| x.u32()));
    info!("{:?}", result.get(jpg_tags::thumb_len).map(|x| x.u32()));

    Ok(())
}
