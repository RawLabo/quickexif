#![allow(dead_code)]
#![allow(unused_imports)]

use log::info;
use std::{collections::HashMap, fs::File};
use quickexif::log_helper::*;

mod adobe_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x010f make
            0x0110 model
            0xc614 make_model
            0xc717 is_converted
            0xc621 color_matrix_1  // for apple pro raw
            0xc622 color_matrix_2 // for normal dng

            0x0112 orientation
            0xc628 white_balance

            0x0111 thumbnail0
            0x0117 thumbnail_len0
            0x828e cfa_pattern0
            0x0144 tile_offsets0
            0x0145 tile_byte_counts0
            0x0142 tile_width0
            0x0143 tile_len0
            0xc61d white_level0
            0xc61a black_level0
            0xc61f crop_xy0
            0xc620 crop_size0
        }
        0 -> 0x014a -> 0 {
            0x0111 thumbnail1
            0x0117 thumbnail_len1
            0x828e cfa_pattern1
            0x0144 tile_offsets1
            0x0145 tile_byte_counts1
            0x0142 tile_width1
            0x0143 tile_len1
            0xc61d white_level1
            0xc61a black_level1
            0xc61f crop_xy1
            0xc620 crop_size1
        }
        0 -> 0x014a -> 100 {
            0x0111 thumbnail2
            0x0117 thumbnail_len2
            0x828e cfa_pattern2
            0x0144 tile_offsets2
            0x0145 tile_byte_counts2
            0x0142 tile_width2
            0x0143 tile_len2
            0xc61d white_level2
            0xc61a black_level2
            0xc61f crop_xy2
            0xc620 crop_size2
        }
        0 -> 0x014a -> 200 {
            0x0111 thumbnail3
            0x0117 thumbnail_len3
            0x828e cfa_pattern3
            0x0144 tile_offsets3
            0x0145 tile_byte_counts3
            0x0142 tile_width3
            0x0143 tile_len3
            0xc61d white_level3
            0xc61a black_level3
            0xc61f crop_xy3
            0xc620 crop_size3
        }
    );
}

#[test]
fn parse_orf() -> LogResult<()> {
    env_logger::init();
    let sample = "tests/samples/sample0.dng";
    let f = q!(File::open(sample));

    let result = q!(quickexif::parse_exif(f, adobe_tags::PATH_LST, None));

    info!("{:?}", result.get(adobe_tags::make).and_then(|x| x.str()));
    info!("{:?}", result.get(adobe_tags::model).and_then(|x| x.str()));
    info!("{:?}", result.get(adobe_tags::make_model).and_then(|x| x.str()));
    info!("{:?}", result.get(adobe_tags::is_converted).and_then(|x| x.str()));
    info!("{:?}", result.get(adobe_tags::color_matrix_1).and_then(|x| x.r64s()));
    info!("{:?}", result.get(adobe_tags::color_matrix_2).and_then(|x| x.r64s()));

    if result.get(adobe_tags::cfa_pattern0).is_some() {
        info!("{:?}", result.get(adobe_tags::thumbnail0).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::thumbnail_len0).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::cfa_pattern0).map(|x| x.raw()));
        info!("{:?}", result.get(adobe_tags::tile_offsets0).and_then(|x| x.u32s()));
        info!("{:?}", result.get(adobe_tags::tile_byte_counts0).and_then(|x| x.u32s()));
        info!("{:?}", result.get(adobe_tags::tile_width0).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::tile_len0).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::white_level0).map(|x| x.u16()));
        info!("{:?}", result.get(adobe_tags::black_level0).map(|x| x.u16()));
        info!("{:?}", result.get(adobe_tags::crop_xy0).and_then(|x| x.r64s()));
        info!("{:?}", result.get(adobe_tags::crop_size0).and_then(|x| x.r64s()));
    }

    if result.get(adobe_tags::cfa_pattern1).is_some() {
        info!("{:?}", result.get(adobe_tags::thumbnail2).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::thumbnail_len2).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::cfa_pattern1).map(|x| x.raw()));
        info!("{:?}", result.get(adobe_tags::tile_offsets1).and_then(|x| x.u32s()));
        info!("{:?}", result.get(adobe_tags::tile_byte_counts1).and_then(|x| x.u32s()));
        info!("{:?}", result.get(adobe_tags::tile_width1).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::tile_len1).map(|x| x.u32()));
        info!("{:?}", result.get(adobe_tags::white_level1).map(|x| x.u16()));
        info!("{:?}", result.get(adobe_tags::black_level1).map(|x| x.u16()));
        info!("{:?}", result.get(adobe_tags::crop_xy1).and_then(|x| x.r64s()));
        info!("{:?}", result.get(adobe_tags::crop_size1).and_then(|x| x.r64s()));
    }

    Ok(())
}