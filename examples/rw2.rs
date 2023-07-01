#![allow(dead_code)]
#![allow(unused_imports)]

use std::{collections::HashMap, fs::File};
use quickexif::log_helper::*;

mod panasonic_tags {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x0002 width
            0x0003 height
            0x0009 cfa_pattern
            0x000a bps
            0x001c black_level_r
            0x001d black_level_g
            0x001e black_level_b
            0x0024 white_balance_r
            0x0025 white_balance_g
            0x0026 white_balance_b
            0x0118 strip
            0x0117 strip_len
            0x002f crop_top
            0x0030 crop_left
            0x0031 crop_bottom
            0x0032 crop_right
            0x0112 orientation
            0x002e thumbnail
        }
        0 -> 0x002e -> 0 {}
        0 -> 0x002e -> 0 -> 0x8769 -> 0 {}
        0 -> 0x002e -> 0 -> 0x8769 -> 0 -> 0x927c -> 0 {
            0x004b cropped_width
            0x004c cropped_height
        }
    );
}

fn main() -> LogResult<()> {
    let sample = "examples/samples/sample0.RW2";
    let f = q!(File::open(sample));

    let result = q!(quickexif::parse_exif(f, panasonic_tags::PATH_LST, None));

    println!("{:?}", result.get(panasonic_tags::width).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::height).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::cfa_pattern).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::bps).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::black_level_r).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::black_level_g).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::black_level_b).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::white_balance_r).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::white_balance_g).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::white_balance_b).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::strip).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::strip_len).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::crop_top).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::crop_left).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::crop_bottom).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::crop_right).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::orientation).map(|x| x.u16()));
    println!("{:?}", result.get(panasonic_tags::cropped_width).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::cropped_height).map(|x| x.u32()));
    println!("{:?}", result.get(panasonic_tags::thumbnail).map(|x| (x.u32(), x.size())));

    Ok(())
}
