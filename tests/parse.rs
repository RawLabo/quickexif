use log::info;
use std::{collections::HashMap, fs::File};

// quickexif::describe_rule!(tiff {
//     0x010f {
//         str + 0 / make
//     }
//     0x0110 {
//         str + 0 / model
//     }
//     0x828e? / cfa_pattern
//     0xc612? / dng_version
//     if dng_version ? {
//         0xc614 {
//             str + 0 / make_model
//         }
//         if cfa_pattern ? {
//             0xc622 { // for normal dng
//                 r64 + 0 / c0
//                 r64 + 1 / c1
//                 r64 + 2 / c2
//                 r64 + 3 / c3
//                 r64 + 4 / c4
//                 r64 + 5 / c5
//                 r64 + 6 / c6
//                 r64 + 7 / c7
//                 r64 + 8 / c8
//             }
//         } else {
//             0xc621 { // for Apple ProRaw
//                 r64 + 0 / c0
//                 r64 + 1 / c1
//                 r64 + 2 / c2
//                 r64 + 3 / c3
//                 r64 + 4 / c4
//                 r64 + 5 / c5
//                 r64 + 6 / c6
//                 r64 + 7 / c7
//                 r64 + 8 / c8
//             }
//         }
//     }
// })

// quickexif::describe_rule!(tiff {
//     0x0112 / orientation
//     0x0201 / preview_offset
//     0x0202 / preview_len
// })

// quickexif::describe_rule!(tiff {
//     0x0112 / orientation
//     0x8769 {
//         0x9102 {
//             r64 + 0 / compressed_bps
//         }
//     }
//     0x014a {
//         0x0103 / compression
//         0x0100 / width
//         0x0101 / height
//         0x0102 / bps
//         0x828e / cfa_pattern
//         0x0111 / strip
//         0x0117 / strip_len
//         0x7010? / tone_curve_addr
//         0xc61f? {
//             u32 + 0 / crop_x
//             u32 + 1 / crop_y
//         }
//         0xc620? {
//             u32 + 0 / crop_w
//             u32 + 1 / crop_h
//         }
//     }
//     0xc634 {
//         sony_decrypt / 0x7200 / 0x7201 / 0x7221 {
//             0x7310 {
//                 u16 + 0 / black_level
//             }
//             0x7312 {
//                 u16 + 0 / white_balance_r
//                 u16 + 1 / white_balance_g
//                 u16 + 3 / white_balance_b
//             }
//             0x787f / legacy_white_level {
//                 u16 + 0 / white_level
//             }
//         }
//     }
// })

macro_rules! gen_tags_mapping {
    [$($path:literal)->* { $($body:tt)* } $($tails:tt)*] => {
        gen_tags_mapping![@path(&[$($path),*],) @defs() @path_index(0; $($body)*) $($tails)*];
    };
    
    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr;) $($path:literal)->* { $($body:tt)* } $($tails:tt)*] => {
        gen_tags_mapping![@path($($p)* &[$($path),*],) @defs($($d)*) @path_index($pi + 1; $($body)*) $($tails)*];
    };

    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr; $tag:literal $id:ident $($inner_tails:tt)*) $($tails:tt)*] => {
        gen_tags_mapping![@path($($p)*) @defs($($d)* pub const $id:&(u16, u16) = &($pi, $tag);) @path_index($pi; $($inner_tails)*) $($tails)*];
    };

    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr;)] => {
        pub const PATH_LST : &[&'static [u16]] = &[$($p)*];
        $($d)*
    }
}

mod SonyTags {
    #![allow(non_upper_case_globals)]
    gen_tags_mapping!(
        0 -> 0xc634 -> 0 {}
        0 -> 0xc634 -> 0 -> 0x7200 -> 0xffff {
            0x7310 black_level
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
        1 {}
    );
}

#[test]
fn parse_arw() -> quickexif::R<()> {
    env_logger::init();
    let sample = "tests/samples/sample0.ARW";
    let f = File::open(sample)?;

    let result = quickexif::parse_exif(f, SonyTags::PATH_LST, Some((0, 1)))?;

    info!("{:?}", result.get(SonyTags::make).and_then(|x| x.str()));
    info!("{:?}", result.get(SonyTags::model).and_then(|x| x.str()));

    info!("{:?}", result.get(SonyTags::orientation).map(|x| x.u16()));
    info!("{:x?}", result.get(SonyTags::preview_offset).map(|x| x.u32()));
    info!("{:?}", result.get(SonyTags::preview_len).map(|x| x.u32()));

    info!("{:?}", result.get(SonyTags::compressed_bps).and_then(|x| x.r64s()));
    info!("{:?}", result.get(SonyTags::compression).map(|x| x.u16()));
    info!("{:?}", result.get(SonyTags::width).map(|x| x.u16()));
    info!("{:?}", result.get(SonyTags::height).map(|x| x.u16()));
    info!("{:?}", result.get(SonyTags::bps).map(|x| x.u16()));
    info!("{:x?}", result.get(SonyTags::cfa_pattern).map(|x| x.raw()));
    info!("{:x?}", result.get(SonyTags::strip).map(|x| x.u32()));
    info!("{:x?}", result.get(SonyTags::strip_len).map(|x| x.u32()));
    info!("{:?}", result.get(SonyTags::tone_curve_addr).and_then(|x| x.u16s()));

    info!("{:?}", result.get(SonyTags::crop_xy).and_then(|x| x.u32s()));
    info!("{:?}", result.get(SonyTags::crop_wh).and_then(|x| x.u32s()));

    info!("{:?}", result.get(SonyTags::black_level).and_then(|x| x.u16s()));

    // let mut counter = HashMap::new();
    // for ((path_index, tag), ifd_item) in result.iter() {
    //     let prev = if let Some(x) = counter.get(&path_index) {
    //         *x
    //     } else {
    //         0
    //     };
    //     counter.insert(path_index, prev + 1);
    // }
    // info!("{:?}", counter);

    Ok(())
}
