use std::{collections::HashMap, fs::File};
use log::info;

#[test]
fn parse_arw() -> quickexif::R<()> {
    env_logger::init();
    let sample = "tests/samples/sample0.ARW";
    let f = File::open(sample)?;
    let path_lst: &[&'static [u8]] = &[
        &[0u8],
        &[0u8, 0x69, 0x87, 0],
        &[0u8, 0x34, 0xc6, 0],
        &[0u8, 0x34, 0xc6, 0, 0x00, 0x72, 0xff],
        &[1u8],
    ];

    let result = quickexif::parse_exif(f, path_lst, Some((2, 3)))?;
    
    let mut counter = HashMap::new();
    for (key, ifd_item) in result.iter() {
        let index = key[0];
        let prev = if let Some(x) = counter.get(&index) {
            *x
        } else {
            0
        };
        counter.insert(index, prev + 1);
    }

    info!("{:?}", counter);
    Ok(())
}

