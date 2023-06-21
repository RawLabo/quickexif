pub(crate) mod util;

use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read, Seek},
};
use util::*;

#[derive(Debug)]
struct IFDItem {
    tag: [u8; 2],
    format: [u8; 2],
    size: [u8; 4],
    value: [u8; 4],
}

struct TiffParser<T: Read + Seek> {
    is_le: bool,
    init_pos: i64,
    reader: BufReader<T>,
}

macro_rules! gen_num_helper {
    ($t:ident, $size:literal) => {
        fn $t(&self, x: [u8; $size]) -> $t {
            if self.is_le {
                $t::from_le_bytes(x)
            } else {
                $t::from_be_bytes(x)
            }
        }
    };
}

impl<T: Read + Seek> TiffParser<T> {
    gen_num_helper!(u32, 4);
    gen_num_helper!(u16, 2);

    fn read_shift<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut ret = [0u8; N];
        self.reader.read_exact(&mut ret)?;
        Ok(ret)
    }
    fn read_no_shift<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let ret = self.read_shift();
        self.reader.seek_relative(N as i64 * -1)?;
        ret
    }
    fn seek_ab(&mut self, loc: u32) -> io::Result<()> {
        let pos = self.reader.stream_position()?;
        self.reader
            .seek_relative(self.init_pos + loc as i64 - pos as i64)
    }
    fn seek_re(&mut self, loc: i64) -> io::Result<()> {
        self.reader.seek_relative(self.init_pos + loc)
    }

    fn new(mut reader: BufReader<T>) -> io::Result<Self> {
        let init_pos = reader.stream_position()?;
        let is_le = {
            let mut header = [0u8; 2];
            reader.read_exact(&mut header)?;
            header == [0x49u8, 0x49]
        };
        Ok(Self {
            is_le,
            init_pos: init_pos as i64,
            reader,
        })
    }

    fn parse_ifd(
        &mut self,
        path: Vec<u8>,
        collector: &mut HashMap<&'static [u8], Vec<IFDItem>>,
    ) -> io::Result<()> {
        let entry_count = {
            let x = self.read_shift::<2>()?;
            self.u16(x)
        };
        self.seek_re(entry_count as i64 * 12)?;
        let next_ifd_offset = {
            let x = self.read_no_shift::<4>()?;
            self.u32(x)
        };
        self.seek_re(entry_count as i64 * -12)?;

        let mut path_deep = path.clone();
        let path_deep_len = path_deep.len();
        path_deep.extend([0u8, 0, 0]);

        for _ in 0..entry_count {
            let tag = self.read_shift::<2>()?;
            let format = self.read_shift::<2>()?;
            let size = self.read_shift::<4>()?;
            let value = self.read_shift::<4>()?;

            if let Some(x) = collector.get_mut(path.as_slice()) {
                x.push(IFDItem {
                    tag,
                    format,
                    size,
                    value,
                });
            }

            if let Some(x) = path_deep.get_mut(path_deep_len..) {
                x[0] = tag[0];
                x[1] = tag[1];
            }
            
            if collector.contains_key(path_deep.as_slice()) {
                let addr = self.u32(value);
                self.seek_ab(addr)?;
                self.parse_ifd(path_deep.clone(), collector)?;
            }
        }

        if next_ifd_offset != 0 {
            self.seek_ab(next_ifd_offset)?;

            let mut next_path = path.clone();
            if let Some(x) = next_path.last_mut() {
                *x += 1;
            }
            self.parse_ifd(next_path, collector)?;
        }

        Ok(())
    }

    fn parse(&mut self, collector: &mut HashMap<&'static [u8], Vec<IFDItem>>) -> io::Result<()> {
        self.seek_re(2)?;
        let ifd_offset = {
            let x = self.read_shift::<4>()?;
            self.u32(x)
        };
        self.seek_ab(ifd_offset)?;
        self.parse_ifd(vec![0], collector)?;

        Ok(())
    }
}

pub fn parse_exif(path: impl AsRef<str>) -> R<()> {
    let f = File::open(path.as_ref())?;
    let reader = BufReader::new(f);
    let mut parser = TiffParser::new(reader)?;

    let deep_tags: [&'static [u8]; 3] = [&[0u8], &[0u8, 0x69, 0x87, 0u8], &[1u8]];
    let mut collector = deep_tags
        .into_iter()
        .map(|x| (x, Vec::with_capacity(20)))
        .collect();

    parser.parse(&mut collector)?;

    for (k, v) in collector.iter() {
        println!("{:#x?} {}", k, v.len());
    }
    // println!("{:#x?}", collector);
    Ok(())
}
