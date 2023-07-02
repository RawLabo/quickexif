#![allow(dead_code)]
#![allow(unused_imports)]

use quickexif::log_helper::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek},
};

mod fuji_tags1 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x0112 orientation
            0x010f make
            0x0110 model
        }
        1 {
            0x0201 thumbnail
            0x0202 thumbnail_len
        }
    );
}

mod fuji_tags2 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 -> 0xf000 -> 0 {
            0xf001 width
            0xf002 height
            0xf003 bps
            0xf007 strip
            0xf008 strip_len
            0xf00a black_level
            0xf00d white_balance
        }
    );
}

fn main() -> LogResult<()> {
    let sample = "examples/samples/sample0.RAF";
    {
        let result = q!(quickexif::parse_exif(
            BufReader::new(q!(File::open(sample))),
            fuji_tags1::PATH_LST,
            None
        ));
        println!("{:?}", result.get(fuji_tags1::orientation).map(|x| x.u16()));
        println!("{:?}", result.get(fuji_tags1::make).and_then(|x| x.str()));
        println!("{:?}", result.get(fuji_tags1::model).and_then(|x| x.str()));
        println!("{:x?}", result.get(fuji_tags1::thumbnail).map(|x| x.u32()));
        println!(
            "{:?}",
            result.get(fuji_tags1::thumbnail_len).map(|x| x.u32())
        );
    }

    {
        let f = q!(File::open(sample));
        let mut reader = BufReader::new(f);

        reader.seek_relative(160 + 4).unwrap();
        loop {
            let mut x = [0u8; 4];
            reader.read_exact(&mut x).unwrap();
            if x == [0x49, 0x49, 0x2a, 0x00] {
                reader.seek_relative(-4).unwrap();
                break;
            }
        }

        let result = q!(quickexif::parse_exif(reader, fuji_tags2::PATH_LST, None));

        println!("{:?}", result.get(fuji_tags2::width).map(|x| x.u32()));
        println!("{:?}", result.get(fuji_tags2::height).map(|x| x.u32()));
        println!("{:?}", result.get(fuji_tags2::bps).map(|x| x.u32()));
        println!("{:?}", result.get(fuji_tags2::strip).map(|x| x.u32()));
        println!("{:?}", result.get(fuji_tags2::strip_len).map(|x| x.u32()));
        println!("{:?}", result.get(fuji_tags2::black_level).and_then(|x| x.u32s()));
        println!("{:?}", result.get(fuji_tags2::white_balance).and_then(|x| x.u32s()));
    }

    Ok(())
}
