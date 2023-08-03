#![allow(dead_code)]
#![allow(unused_imports)]

use phf::phf_map;
use std::{
    collections::HashMap,
    io::{BufReader, Read, Seek},
};

use log::info;

erreport::gen_trait_to_report!();
use erreport::Report;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Tiff header: {0:#x?}")]
    InvalidTiffHeader([u8; 2]),
    #[error("Part({0}) is not defined for this file type")]
    PartNotDefined(u8),
}

#[derive(Debug)]
pub struct IFDItem {
    is_le: bool,
    tag: u16,
    format: [u8; 2],
    size: [u8; 4],
    value: [u8; 4],
    actual_value: Option<Box<[u8]>>,
}

impl IFDItem {
    pub fn raw(&self) -> &[u8] {
        match self.actual_value.as_ref() {
            Some(x) => x,
            None => &self.value,
        }
    }
    pub fn size(&self) -> u32 {
        if self.is_le {
            u32::from_le_bytes(self.size)
        } else {
            u32::from_be_bytes(self.size)
        }
    }
    pub fn u16(&self) -> u16 {
        let v = [self.value[0], self.value[1]];
        if self.is_le {
            u16::from_le_bytes(v)
        } else {
            u16::from_be_bytes(v)
        }
    }
    pub fn u32(&self) -> u32 {
        if self.is_le {
            u32::from_le_bytes(self.value)
        } else {
            u32::from_be_bytes(self.value)
        }
    }
    pub fn str(&self) -> Option<&str> {
        self.actual_value.as_ref().and_then(|bytes| {
            std::str::from_utf8(bytes)
                .ok()
                .and_then(|x| x.strip_suffix('\0'))
        })
    }
    pub fn u16s(&self) -> Option<Box<[u16]>> {
        self.actual_value.as_ref().map(|bytes| {
            bytes
                .chunks_exact(2)
                .map(|x| {
                    let v: [u8; 2] = [x[0], x[1]];
                    if self.is_le {
                        u16::from_le_bytes(v)
                    } else {
                        u16::from_be_bytes(v)
                    }
                })
                .collect()
        })
    }
    pub fn u32s(&self) -> Option<Box<[u32]>> {
        self.actual_value.as_ref().map(|bytes| {
            bytes
                .chunks_exact(4)
                .map(|x| {
                    let v: [u8; 4] = [x[0], x[1], x[2], x[3]];
                    if self.is_le {
                        u32::from_le_bytes(v)
                    } else {
                        u32::from_be_bytes(v)
                    }
                })
                .collect()
        })
    }
    pub fn r64s(&self) -> Option<Box<[f64]>> {
        self.actual_value.as_ref().map(|bytes| {
            bytes
                .chunks_exact(8)
                .map(|x| {
                    let left: [u8; 4] = [x[0], x[1], x[2], x[3]];
                    let right: [u8; 4] = [x[4], x[5], x[6], x[7]];

                    if self.is_le {
                        i32::from_le_bytes(left) as f64 / u32::from_le_bytes(right) as f64
                    } else {
                        i32::from_be_bytes(left) as f64 / u32::from_be_bytes(right) as f64
                    }
                })
                .collect()
        })
    }
}

struct TiffParser<T: Read + Seek> {
    is_le: bool,
    addr_offset: i32, // offset for actual value address, useful for internal tiff blocks
    reader: BufReader<T>,
    path_map: HashMap<&'static [u16], u16>,
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
macro_rules! to_bytes {
    ($x:expr, $is_le:expr) => {{
        if $is_le {
            $x.to_le_bytes()
        } else {
            $x.to_be_bytes()
        }
    }};
}

/// first 4 bytes => (shift N bytes, needs to add makernotes' offset)
static MAKERNOTES_HEADER_SIZE: phf::Map<[u8; 4], (i64, Option<i32>)> = phf_map! {
    [0x50, 0x61, 0x6e, 0x61] => (12, None), // panasonic
    [0x4f, 0x4c, 0x59, 0x4d] => (12, Some(0)), // olympus
    [0x4e, 0x69, 0x6b, 0x6f] => (18, Some(10)), // nikon
};

type Collector = HashMap<(u16, u16), IFDItem>;

impl<T: Read + Seek> TiffParser<T> {
    gen_num_helper!(u32, 4);
    gen_num_helper!(u16, 2);

    fn sony_decrypt(&self, data: &[u8], mut key: u32) -> Vec<u8> {
        let mut pad = [0u32; 128];
        for item in pad.iter_mut().take(4) {
            key = key.wrapping_mul(48828125).wrapping_add(1);
            *item = key;
        }
        pad[3] = pad[3] << 1 | (pad[0] ^ pad[2]) >> 31;
        for i in 4..127 {
            pad[i] = (pad[i - 4] ^ pad[i - 2]) << 1 | (pad[i - 3] ^ pad[i - 1]) >> 31;
        }
        for item in pad.iter_mut().take(127) {
            *item = item.swap_bytes();
        }
        data.chunks_exact(4)
            .map(|x| self.u32(x.try_into().unwrap()))
            .zip(127..)
            .flat_map(|(x, p)| {
                pad[p & 127] = pad[(p + 1) & 127] ^ pad[(p + 65) & 127];
                to_bytes!(x ^ pad[p & 127], self.is_le)
            })
            .collect::<Vec<u8>>()
    }

    fn read_to_vec(&mut self, bytes_count: usize) -> Result<Vec<u8>, Report> {
        let mut ret = vec![0u8; bytes_count];
        self.reader.read_exact(&mut ret).to_report()?;
        Ok(ret)
    }
    fn read_shift<const N: usize>(&mut self) -> Result<[u8; N], Report> {
        let mut ret = [0u8; N];
        self.reader.read_exact(&mut ret).to_report()?;
        Ok(ret)
    }
    fn read_no_shift<const N: usize>(&mut self) -> Result<[u8; N], Report> {
        let ret = self.read_shift();
        self.reader.seek_relative(N as i64 * -1).to_report()?;
        ret
    }
    fn seek_ab(&mut self, loc: u32) -> Result<(), Report> {
        let pos = self.reader.stream_position().to_report()?;
        self.reader
            .seek_relative(loc as i64 - pos as i64 + self.addr_offset as i64)
            .to_report()?;
        Ok(())
    }
    fn recover_pos(&mut self, loc: u64) -> Result<(), Report> {
        let pos = self.reader.stream_position().to_report()?;
        self.reader
            .seek_relative(loc as i64 - pos as i64)
            .to_report()?;
        Ok(())
    }

    fn seek_re(&mut self, loc: i64) -> Result<(), Report> {
        self.reader.seek_relative(loc).to_report()?;
        Ok(())
    }

    fn new(
        mut reader: BufReader<T>,
        path_lst: impl AsRef<[&'static [u16]]>,
    ) -> Result<Self, Report> {
        let init_pos = reader.stream_position().to_report()?;
        let addr_offset = init_pos as i32 + {
            // jpg detect
            let mut header = [0u8; 2];
            reader.read_exact(&mut header).to_report()?;
            if header == [0xff, 0xd8] {
                let mut header = [0u8; 2];
                reader.read_exact(&mut header).to_report()?;

                if header == [0xff, 0xe0] {
                    // is JFIF
                    reader.seek_relative(26).to_report()?;
                    30
                } else {
                    // is EXIF
                    reader.seek_relative(8).to_report()?;
                    12
                }
            } else {
                reader.seek_relative(-2).to_report()?;
                0
            }
        };

        let is_le = {
            let mut header = [0u8; 2];
            reader.read_exact(&mut header).to_report()?;
            reader.seek_relative(-2).to_report()?;
            match header {
                [0x49, 0x49] => true,
                [0x4d, 0x4d] => false,
                _ => Err(Error::InvalidTiffHeader(header)).to_report()?,
            }
        };

        let path_map = path_lst
            .as_ref()
            .into_iter()
            .enumerate()
            .map(|(i, x)| (*x, i as u16))
            .collect();

        Ok(Self {
            is_le,
            addr_offset,
            reader,
            path_map,
        })
    }

    fn check_actual_value(
        &mut self,
        format: [u8; 2],
        size: [u8; 4],
        addr: [u8; 4],
    ) -> Result<Option<Box<[u8]>>, Report> {
        let format = self.u16(format);
        let format_size = match format {
            0x0001 => 1u32, // u8
            0x0002 => 1,    // string
            0x0003 => 2,    // u16
            0x0004 => 4,    // u32
            0x0005 => 8,
            0x0006 => 1,
            0x0007 => 1,
            0x0008 => 2,
            0x0009 => 4,
            0x000a => 8,
            0x000b => 4,
            0x000c => 8,
            0x000d => 4,
            0x000e => 8,
            _ => 1,
        };
        let total_size = self.u32(size) * format_size;
        if total_size > 4 || format == 0x0002 {
            let addr = self.u32(addr);
            let pos = self.reader.stream_position().to_report()?;
            self.seek_ab(addr).to_report()?;
            let actual_value = self.read_to_vec(total_size as usize).to_report()?;
            self.recover_pos(pos).to_report()?;
            Ok(Some(actual_value.into()))
        } else {
            Ok(None)
        }
    }

    fn parse_ifd(&mut self, path: Vec<u16>, collector: &mut Collector) -> Result<(), Report> {
        let entry_count = {
            let x = self.read_shift::<2>().to_report()?;
            self.u16(x)
        };

        let mut path_deep = path.clone();
        let path_deep_len = path_deep.len();
        path_deep.extend([0u16, 0]);

        let mut dig_deep = vec![];
        for _ in 0..entry_count {
            let tag = self.read_shift::<2>().to_report()?;
            let tag = self.u16(tag);

            let format = self.read_shift::<2>().to_report()?;
            let size = self.read_shift::<4>().to_report()?;
            let value = self.read_shift::<4>().to_report()?;
            let actual_value = self.check_actual_value(format, size, value).to_report()?;

            let ifd_item = IFDItem {
                is_le: self.is_le,
                tag,
                format,
                size,
                value,
                actual_value,
            };

            // switch to the current tag
            if let Some(x) = path_deep.get_mut(path_deep_len..) {
                x[0] = tag;
            }
            // save addr and path for later deeper digging
            if self.path_map.contains_key(path_deep.as_slice()) {
                if let (Some(addrs), 0x0004) = (ifd_item.u32s(), self.u16(format)) {
                    dig_deep.extend(addrs.into_iter().enumerate().map(|(i, addr)| {
                        let mut path = path_deep.clone();
                        if let Some(last) = path.last_mut() {
                            *last = (i * 100) as u16; // set path ifd id to 0, 100, 200, 300
                        }
                        (*addr, path)
                    }))
                } else {
                    let addr = self.u32(value);
                    dig_deep.push((addr, path_deep.clone()));
                }
            }

            if let Some(&path_index) = self.path_map.get(path.as_slice()) {
                collector.insert((path_index, tag), ifd_item);
            }
        }

        let next_ifd_offset = {
            let x = self.read_shift::<4>().to_report()?;
            self.u32(x)
        };
        if next_ifd_offset != 0 && next_ifd_offset < 0xffffff {
            self.seek_ab(next_ifd_offset).to_report()?;

            let mut next_path = path.clone();
            if let Some(x) = next_path.last_mut() {
                *x += 1;
            }
            self.parse_ifd(next_path, collector).to_report()?;
        }

        let addr_offset = self.addr_offset;
        for (addr, path) in dig_deep {
            self.addr_offset = addr_offset; // offset recover
            self.seek_ab(addr).to_report()?;

            // detect if is jpg header
            if self.read_no_shift::<2>().to_report()? == [0xff, 0xd8] {
                self.seek_re(12).to_report()?; // pass JPEG header
                self.addr_offset = self.reader.stream_position().to_report()? as i32;
                self.shift_from_tiff_header().to_report()?;
            }
            // detect if is makernotes
            let check = self.read_no_shift::<4>().to_report()?;
            if let Some(&(shift, addr_offset)) = MAKERNOTES_HEADER_SIZE.get(&check) {
                if let Some(offset) = addr_offset {
                    self.addr_offset += self.reader.stream_position().to_report()? as i32 + offset;
                }
                self.seek_re(shift).to_report()?;
            }

            self.parse_ifd(path, collector).to_report()?;
        }

        Ok(())
    }

    fn shift_from_tiff_header(&mut self) -> Result<(), Report> {
        self.seek_re(4).to_report()?;
        let ifd_offset = {
            let x = self.read_shift::<4>().to_report()?;
            self.u32(x)
        };
        self.seek_ab(ifd_offset).to_report()?;
        Ok(())
    }
    fn parse(&mut self) -> Result<Collector, Report> {
        let mut result = HashMap::new();

        self.shift_from_tiff_header().to_report()?;

        self.parse_ifd(vec![0], &mut result).to_report()?;

        Ok(result)
    }
    fn parse_sony_sr2private(
        &mut self,
        sr2private_index: u16, // the index of sr2private path in path_map
        path: Vec<u16>,
        collector: &mut Collector,
    ) -> Result<(), Report> {
        match (
            collector.get(&(sr2private_index, 0x7200)),
            collector.get(&(sr2private_index, 0x7201)),
            collector.get(&(sr2private_index, 0x7221)),
        ) {
            (Some(offset_ifd), Some(length_ifd), Some(key_ifd)) => {
                let offset = self.u32(offset_ifd.value);
                let length = self.u32(length_ifd.value);
                let key = self.u32(key_ifd.value);

                self.seek_ab(offset).to_report()?;
                let sr2private_bytes = self.read_to_vec(length as usize).to_report()?;
                let decrypted = self.sony_decrypt(&sr2private_bytes, key);
                let mut new_parser = TiffParser {
                    is_le: self.is_le,
                    addr_offset: -(offset as i32),
                    reader: BufReader::new(std::io::Cursor::new(decrypted)),
                    path_map: self.path_map.clone(),
                };
                new_parser.parse_ifd(path, collector).to_report()?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// The return data contains (exif_info_hashmap, is_little_endian_marker)
pub fn parse_exif<T: Read + Seek>(
    reader: BufReader<T>,
    path_dig: &[&'static [u16]],
    sony_decrypt_index: Option<(u16, usize)>, // (sr2private_path_index, sr2private_offset_path_index)
) -> Result<(Collector, bool), Report> {
    let mut parser = TiffParser::new(reader, path_dig).to_report()?;
    let mut result = parser.parse().to_report()?;

    if let Some((sr2private_path_index, sr2private_offset_path_index)) = sony_decrypt_index {
        parser
            .parse_sony_sr2private(
                sr2private_path_index,
                path_dig[sr2private_offset_path_index].to_vec(),
                &mut result,
            )
            .to_report()?;
    }

    Ok((result, parser.is_le))
}

fn seek_tiff_header<T: Read + Seek>(reader: &mut BufReader<T>) -> Result<(), Report> {
    loop {
        let mut x = [0u8; 4];
        reader.read_exact(&mut x).to_report()?;
        if x == [0x49, 0x49, 0x2a, 0x00] {
            reader.seek_relative(-4).to_report()?;
            break Ok(());
        }
    }
}

fn seek_cr3_cmt<T: Read + Seek>(reader: &mut BufReader<T>, no: u8) -> Result<(), Report> {
    loop {
        let mut x = [0u8; 4];
        reader.read_exact(&mut x).to_report()?;
        if x == [0x43, 0x4d, 0x54, no] {
            break Ok(());
        }
    }
}
fn seek_cr3_header<T: Read + Seek>(reader: &mut BufReader<T>, index: i8) -> Result<(), Report> {
    reader.seek_relative(0x1a00002i64).to_report()?;
    let mut curr = -1i8;
    while curr < index {
        let mut x = [0u8; 4];
        reader.read_exact(&mut x).to_report()?;
        if x == [0x7c, 0x92, 0, 0] {
            curr += 1;
        }
    }
    Ok(())
}

pub fn seek_header_cr3<T: Read + Seek>(reader: &mut BufReader<T>, part: u8) -> Result<(), Report> {
    match part {
        0 => {
            seek_cr3_cmt(reader, 0x31).to_report()?;
        }
        1 => {
            seek_cr3_cmt(reader, 0x32).to_report()?;
        }
        2 => {
            seek_cr3_cmt(reader, 0x33).to_report()?;
        }
        3 => {
            seek_cr3_header(reader, 0).to_report()?;
        }
        4 => {
            seek_cr3_header(reader, 1).to_report()?;
        }
        _ => Err(Error::PartNotDefined(part)).to_report()?,
    }
    Ok(())
}

pub fn seek_header_raf<T: Read + Seek>(reader: &mut BufReader<T>, part: u8) -> Result<(), Report> {
    match part {
        0 => {
            // cut first 148 bytes
            reader.seek_relative(148).to_report()?;
        }
        1 => {
            // jump to the next tiff header
            reader.seek_relative(160 + 4).to_report()?;
            seek_tiff_header(reader).to_report()?;
        }
        _ => Err(Error::PartNotDefined(part)).to_report()?,
    }

    Ok(())
}

#[macro_export]
macro_rules! gen_tags_info {
    [$($path:literal)->* { $($body:tt)* } $($tails:tt)*] => {
        gen_tags_info![@path(&[$($path),*],) @defs() @path_index(0; $($body)*) $($tails)*];
    };

    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr;) $($path:literal)->* { $($body:tt)* } $($tails:tt)*] => {
        gen_tags_info![@path($($p)* &[$($path),*],) @defs($($d)*) @path_index($pi + 1; $($body)*) $($tails)*];
    };

    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr; $tag:literal $id:ident $($inner_tails:tt)*) $($tails:tt)*] => {
        gen_tags_info![@path($($p)*) @defs($($d)* pub const $id:&(u16, u16) = &($pi, $tag);) @path_index($pi; $($inner_tails)*) $($tails)*];
    };

    [@path($($p:tt)*) @defs($($d:tt)*) @path_index($pi:expr;)] => {
        pub const PATH_LST : &[&'static [u16]] = &[$($p)*];
        $($d)*
    }
}
