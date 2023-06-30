#![allow(dead_code)]
#![allow(unused_imports)]

pub mod log_helper;

use log::info;
use log_helper::*;
use std::{
    collections::HashMap,
    io::{BufReader, Read, Seek},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Tiff header: {0:#x?}")]
    InvalidTiffHeader([u8; 2])
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
            None => &self.value 
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
    pub fn str(&self) -> Option<String> {
        self.actual_value.as_ref().map(|bytes| {
            bytes
                .iter()
                .take(bytes.len() - 1)
                .map(|&x| x as char)
                .collect()
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

    fn read_to_vec(&mut self, bytes_count: usize) -> LogResult<Vec<u8>> {
        let mut ret = vec![0u8; bytes_count];
        q!(self.reader.read_exact(&mut ret));
        Ok(ret)
    }
    fn read_shift<const N: usize>(&mut self) -> LogResult<[u8; N]> {
        let mut ret = [0u8; N];
        q!(self.reader.read_exact(&mut ret));
        Ok(ret)
    }
    fn read_no_shift<const N: usize>(&mut self) -> LogResult<[u8; N]> {
        let ret = self.read_shift();
        q!(self.reader.seek_relative(N as i64 * -1));
        ret
    }
    fn seek_ab(&mut self, loc: u32) -> LogResult<()> {
        let pos = q!(self.reader.stream_position());
        q!(self
            .reader
            .seek_relative(loc as i64 - pos as i64 + self.addr_offset as i64));
        Ok(())
    }
    fn recover_pos(&mut self, loc: u64) -> LogResult<()> {
        let pos = q!(self.reader.stream_position());
        q!(self
            .reader
            .seek_relative(loc as i64 - pos as i64));
        Ok(())
    }

    fn seek_re(&mut self, loc: i64) -> LogResult<()> {
        q!(self.reader.seek_relative(loc));
        Ok(())
    }

    fn new(mut reader: BufReader<T>, path_lst: impl AsRef<[&'static [u16]]>) -> LogResult<Self> {
        // jpg detect
        let addr_shift = {
            let mut header = [0u8; 2];
            q!(reader.read_exact(&mut header));
            if header == [0xff, 0xd8] {
                let mut header = [0u8; 2];
                q!(reader.read_exact(&mut header));

                if header == [0xff, 0xe0] { // is JFIF
                    q!(reader.seek_relative(26));
                    30
                } else { // is EXIF
                    q!(reader.seek_relative(8));
                    12
                }
            } else {
                q!(reader.seek_relative(-2));
                0
            }
        };

        let is_le = {
            let mut header = [0u8; 2];
            q!(reader.read_exact(&mut header));
            q!(reader.seek_relative(-2));
            match header {
                [0x49, 0x49] => true,
                [0x4d, 0x4d] => false,
                _ => q!(Err(Error::InvalidTiffHeader(header)))
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
            addr_offset: addr_shift,
            reader,
            path_map,
        })
    }

    fn check_actual_value(
        &mut self,
        format: [u8; 2],
        size: [u8; 4],
        addr: [u8; 4],
    ) -> LogResult<Option<Box<[u8]>>> {
        let format_size = match format {
            [0x01, 0] => 1u32, // u8
            [0x02, 0] => 1, // string
            [0x03, 0] => 2, // u16
            [0x04, 0] => 4, // u32
            [0x05, 0] => 8,
            [0x06, 0] => 1,
            [0x07, 0] => 1,
            [0x08, 0] => 2,
            [0x09, 0] => 4,
            [0x0a, 0] => 8,
            [0x0b, 0] => 4,
            [0x0c, 0] => 8,
            [0x0d, 0] => 4,
            [0x0e, 0] => 8,
            _ => 1,
        };
        let total_size = self.u32(size) * format_size;
        if total_size > 4 || format[0] == 0x02 {
            let addr = self.u32(addr);
            let pos = q!(self.reader.stream_position());
            q!(self.seek_ab(addr));
            let actual_value = q!(self.read_to_vec(total_size as usize));
            q!(self.recover_pos(pos));
            Ok(Some(actual_value.into()))
        } else {
            Ok(None)
        }
    }

    fn parse_ifd(&mut self, path: Vec<u16>, collector: &mut Collector) -> LogResult<()> {
        let entry_count = {
            let x = q!(self.read_shift::<2>());
            self.u16(x)
        };

        let mut path_deep = path.clone();
        let path_deep_len = path_deep.len();
        path_deep.extend([0u16, 0]);

        let mut dig_deep = vec![];
        for _ in 0..entry_count {
            let tag = q!(self.read_shift::<2>());
            let tag = self.u16(tag);

            let format = q!(self.read_shift::<2>());
            let size = q!(self.read_shift::<4>());
            let value = q!(self.read_shift::<4>());
            let actual_value = q!(self.check_actual_value(format, size, value));

            if let Some(&path_index) = self.path_map.get(path.as_slice()) {
                collector.insert(
                    (path_index, tag),
                    IFDItem {
                        is_le: self.is_le,
                        tag,
                        format,
                        size,
                        value,
                        actual_value,
                    },
                );
            }

            // save addr and path for later deeper digging
            if let Some(x) = path_deep.get_mut(path_deep_len..) {
                x[0] = tag;
            }
            if self.path_map.contains_key(path_deep.as_slice()) {
                let addr = self.u32(value);
                dig_deep.push((addr, path_deep.clone()));
            }
        }

        let next_ifd_offset = {
            let x = q!(self.read_shift::<4>());
            self.u32(x)
        };
        if next_ifd_offset != 0 && next_ifd_offset < 0xffffff {
            q!(self.seek_ab(next_ifd_offset));

            let mut next_path = path.clone();
            if let Some(x) = next_path.last_mut() {
                *x += 1;
            }
            q!(self.parse_ifd(next_path, collector));
        }

        let addr_offset = self.addr_offset;
        for (addr, path) in dig_deep {
            self.addr_offset = addr_offset; // offset recover
            q!(self.seek_ab(addr));

            // detect if is jpg header
            if q!(self.read_no_shift::<2>()) == [0xff, 0xd8] {
                q!(self.seek_re(12)); // pass JPEG header
                self.addr_offset = q!(self.reader.stream_position()) as i32;
                q!(self.shift_from_tiff_header());
            }
            // detect if is panasonic makernotes
            if q!(self.read_no_shift::<4>()) == [0x50, 0x61, 0x6e, 0x61] {
                q!(self.seek_re(12)); // pass makernotes header
            }

            q!(self.parse_ifd(path, collector));
        }

        Ok(())
    }

    fn shift_from_tiff_header(&mut self) -> LogResult<()> {
        q!(self.seek_re(4));
        let ifd_offset = {
            let x = q!(self.read_shift::<4>());
            self.u32(x)
        };
        q!(self.seek_ab(ifd_offset));
        Ok(())
    }
    fn parse(&mut self) -> LogResult<Collector> {
        let mut result = HashMap::new();

        q!(self.shift_from_tiff_header());

        q!(self.parse_ifd(vec![0], &mut result));

        Ok(result)
    }
    fn parse_sony_sr2private(
        &mut self,
        sr2private_index: u16, // the index of sr2private path in path_map
        path: Vec<u16>,
        collector: &mut Collector,
    ) -> LogResult<()> {
        match (
            collector.get(&(sr2private_index, 0x7200)),
            collector.get(&(sr2private_index, 0x7201)),
            collector.get(&(sr2private_index, 0x7221)),
        ) {
            (Some(offset_ifd), Some(length_ifd), Some(key_ifd)) => {
                let offset = self.u32(offset_ifd.value);
                let length = self.u32(length_ifd.value);
                let key = self.u32(key_ifd.value);

                q!(self.seek_ab(offset));
                let sr2private_bytes = q!(self.read_to_vec(length as usize));
                let decrypted = self.sony_decrypt(&sr2private_bytes, key);
                let mut new_parser = TiffParser {
                    is_le: self.is_le,
                    addr_offset: -(offset as i32),
                    reader: BufReader::new(std::io::Cursor::new(decrypted)),
                    path_map: self.path_map.clone(),
                };
                q!(new_parser.parse_ifd(path, collector));
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn parse_exif<T: Read + Seek>(
    input: T,
    path_dig: &[&'static [u16]],
    sony_decrypt_index: Option<(u16, usize)>, // (sr2private_path_index, sr2private_offset_path_index)
) -> LogResult<Collector> {
    let reader = BufReader::new(input);

    let mut parser = q!(TiffParser::new(reader, path_dig));
    let mut result = q!(parser.parse());

    if let Some((sr2private_path_index, sr2private_offset_path_index)) = sony_decrypt_index {
        q!(parser.parse_sony_sr2private(
            sr2private_path_index,
            path_dig[sr2private_offset_path_index].to_vec(),
            &mut result,
        ));
    }

    Ok(result)
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
