#![allow(dead_code)]
#![allow(unused_imports)]

use log::info;
use std::{collections::HashMap, fs::File};
use quickexif::log_helper::*;

mod olympus_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x0112 orientation
            0x0100 width
            0x0101 height
            0x0111 strip
            0x0117 strip_len
        }
        0 -> 0x8769 -> 0 {
            0xa302 cfa_pattern
        }
        0 -> 0x8769 -> 0 -> 0x927c -> 0 {}
        0 -> 0x8769 -> 0 -> 0x927c -> 0 -> 0x2040 -> 0 {
            0x0611 bps
            0x0612 crop_left
            0x0613 crop_top
            0x0614 crop_width
            0x0615 crop_height
            0x0100 white_balance
            0x0600 black_level
        }
    );
}

#[test]
fn parse_orf() -> LogResult<()> {
    env_logger::init();
    let sample = "tests/samples/sample0.ORF";
    let f = q!(File::open(sample));

    let result = q!(quickexif::parse_exif(f, olympus_tags::PATH_LST, None));

    info!("{:?}", result.get(olympus_tags::orientation).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::width).map(|x| x.u32()));
    info!("{:?}", result.get(olympus_tags::height).map(|x| x.u32()));
    info!("{:?}", result.get(olympus_tags::strip).map(|x| x.u32()));
    info!("{:?}", result.get(olympus_tags::strip_len).map(|x| x.u32()));
    info!("{:?}", result.get(olympus_tags::cfa_pattern).map(|x| x.raw()));
    info!("{:?}", result.get(olympus_tags::bps).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::crop_left).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::crop_top).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::crop_width).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::crop_height).map(|x| x.u16()));
    info!("{:?}", result.get(olympus_tags::white_balance).and_then(|x| x.u16s()));
    info!("{:?}", result.get(olympus_tags::black_level).and_then(|x| x.u16s()));

    Ok(())
}
