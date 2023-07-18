#![allow(dead_code)]
#![allow(unused_imports)]

use std::{collections::HashMap, fs::File, io::{BufReader, Seek}};

mod cr3_tags1 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x010f make
            0x0110 model
            0x0112 orientation
            0x0132 last
        }
    );
}

mod cr3_tags2 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0xa002 width
            0xa003 height
        }
    );
}

mod cr3_tags3 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x0038 battery
        }
    );
}

mod cr3_tags4 {
    #![allow(non_upper_case_globals)]
    use quickexif::gen_tags_info;

    gen_tags_info!(
        0 {
            0x4001 colordata
        }
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = "examples/samples/sample0.CR3";
    {
        let mut reader = BufReader::new(File::open(sample)?);
        quickexif::seek_header_cr3(&mut reader, 0)?;
        
        let result = quickexif::parse_exif(reader, cr3_tags1::PATH_LST, None)?;
    
        println!("{:?}", result.get(cr3_tags1::make).and_then(|x| x.str()));
        println!("{:?}", result.get(cr3_tags1::model).and_then(|x| x.str()));
        println!("{:?}", result.get(cr3_tags1::orientation).map(|x| x.u16()));
    }
    {
        let mut reader = BufReader::new(File::open(sample)?);
        quickexif::seek_header_cr3(&mut reader, 1)?;

        let result = quickexif::parse_exif(reader, cr3_tags2::PATH_LST, None)?;

        println!("{:?}", result.get(cr3_tags2::width).map(|x| x.u32()));
        println!("{:?}", result.get(cr3_tags2::height).map(|x| x.u32()));
    }
    {
        let mut reader = BufReader::new(File::open(sample)?);
        quickexif::seek_header_cr3(&mut reader, 2)?;

        let result = quickexif::parse_exif(reader, cr3_tags3::PATH_LST, None)?;

        println!("{:?}", result.get(cr3_tags3::battery).and_then(|x| x.str()));
    }
    {
        let mut reader = BufReader::new(File::open(sample)?);
        quickexif::seek_header_cr3(&mut reader, 4)?;

        let result = quickexif::parse_exif(reader, cr3_tags4::PATH_LST, None)?;

        println!("{:?}", result.get(cr3_tags4::colordata).map(|x| x.raw()));
    }

    Ok(())
}
