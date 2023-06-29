#![allow(dead_code)]
#![allow(unused_imports)]

use log::info;
use std::{collections::HashMap, fs::File};
use quickexif::log_helper::*;

mod sony_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 -> 0xc634 -> 0 {}
        0 -> 0xc634 -> 0 -> 0x7200 -> 0xffff {
            0x7310 black_level
            0x7312 white_balance
            0x787f white_level
        }
        0 {
            0x010f make
            0x0110 model
            0x0112 orientation
            0x0201 preview_offset
            0x0202 preview_len
        }
        0 -> 0x8769 -> 0 {
            0x9102 compressed_bps
        }
        0 -> 0x014a -> 0 {
            0x0103 compression
            0x0100 width
            0x0101 height
            0x0102 bps
            0x828e cfa_pattern
            0x0111 strip
            0x0117 strip_len
            0x7010 tone_curve_addr
            0xc61f crop_xy
            0xc620 crop_wh
        }
    );
}

#[test]
fn parse_arw() -> LogResult<()> {
    env_logger::init();
    let sample = "tests/samples/sample1.ARW";
    let f = q!(File::open(sample));

    let result = q!(quickexif::parse_exif(f, sony_tags::PATH_LST, Some((0, 1))));

    info!("{:?}", result.get(sony_tags::make).and_then(|x| x.str()));
    info!("{:?}", result.get(sony_tags::model).and_then(|x| x.str()));

    info!("{:?}", result.get(sony_tags::orientation).map(|x| x.u16()));
    info!("{:x?}", result.get(sony_tags::preview_offset).map(|x| x.u32()));
    info!("{:?}", result.get(sony_tags::preview_len).map(|x| x.u32()));

    info!("{:?}", result.get(sony_tags::compressed_bps).and_then(|x| x.r64s()));
    info!("{:?}", result.get(sony_tags::compression).map(|x| x.u16()));
    info!("{:?}", result.get(sony_tags::width).map(|x| x.u16()));
    info!("{:?}", result.get(sony_tags::height).map(|x| x.u16()));
    info!("{:?}", result.get(sony_tags::bps).map(|x| x.u16()));
    info!("{:x?}", result.get(sony_tags::cfa_pattern).map(|x| x.raw()));
    info!("{:x?}", result.get(sony_tags::strip).map(|x| x.u32()));
    info!("{:x?}", result.get(sony_tags::strip_len).map(|x| x.u32()));
    info!("{:?}", result.get(sony_tags::tone_curve_addr).and_then(|x| x.u16s()));

    info!("{:?}", result.get(sony_tags::crop_xy).and_then(|x| x.u32s()));
    info!("{:?}", result.get(sony_tags::crop_wh).and_then(|x| x.u32s()));

    info!("{:?}", result.get(sony_tags::black_level).and_then(|x| x.u16s()));

    info!("{:?}", result.get(sony_tags::white_balance).and_then(|x| x.u16s()));
    info!("{:?}", result.get(sony_tags::white_level).and_then(|x| x.u16s()));

    Ok(())
}
