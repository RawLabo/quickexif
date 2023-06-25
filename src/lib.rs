pub(crate) mod util;

use log::info;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read, Seek},
};
pub use util::R;
use util::*;

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
    pub fn raw(&self) -> [u8; 4] {
        self.value
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
    init_pos: i64,
    addr_shift: i32,  // shift for actual value address
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

    fn read_to_vec(&mut self, bytes_count: usize) -> io::Result<Vec<u8>> {
        let mut ret = vec![0u8; bytes_count];
        self.reader.read_exact(&mut ret)?;
        Ok(ret)
    }
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

    fn new(mut reader: BufReader<T>, path_lst: impl AsRef<[&'static [u16]]>) -> io::Result<Self> {
        let init_pos = reader.stream_position()?;
        let is_le = {
            let mut header = [0u8; 2];
            reader.read_exact(&mut header)?;
            header == [0x49u8, 0x49]
        };

        let path_map = path_lst
            .as_ref()
            .into_iter()
            .enumerate()
            .map(|(i, x)| (*x, i as u16))
            .collect();

        Ok(Self {
            is_le,
            init_pos: init_pos as i64,
            addr_shift: 0,
            reader,
            path_map,
        })
    }

    fn check_actual_value(
        &mut self,
        format: [u8; 2],
        size: [u8; 4],
        addr: [u8; 4],
    ) -> io::Result<Option<Box<[u8]>>> {
        let format_size = match format {
            [0x01, 0] => 1u32,
            [0x02, 0] => 1, // string
            [0x03, 0] => 2,
            [0x04, 0] => 4,
            [0x05, 0] => 8,
            [0x06, 0] => 1,
            [0x07, 0] => 0,
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
            let addr = (self.u32(addr) as i32 + self.addr_shift) as u32;
            let pos = self.reader.stream_position()?;
            self.seek_ab(addr)?;
            let actual_value = self.read_to_vec(total_size as usize)?;
            self.seek_ab(pos as u32)?;
            Ok(Some(actual_value.into()))
        } else {
            Ok(None)
        }
    }

    fn parse_ifd(&mut self, path: Vec<u16>, collector: &mut Collector) -> io::Result<()> {
        let entry_count = {
            let x = self.read_shift::<2>()?;
            self.u16(x)
        };

        let mut path_deep = path.clone();
        let path_deep_len = path_deep.len();
        path_deep.extend([0u16, 0]);

        let mut dig_deep = vec![];
        for _ in 0..entry_count {
            let tag = self.read_shift::<2>()?;
            let tag = self.u16(tag);

            let format = self.read_shift::<2>()?;
            let size = self.read_shift::<4>()?;
            let value = self.read_shift::<4>()?;
            let actual_value = self.check_actual_value(format, size, value)?;

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
            let x = self.read_shift::<4>()?;
            self.u32(x)
        };
        if next_ifd_offset != 0 && next_ifd_offset < 0xffffff {
            self.seek_ab(next_ifd_offset)?;

            let mut next_path = path.clone();
            if let Some(x) = next_path.last_mut() {
                *x += 1;
            }
            self.parse_ifd(next_path, collector)?;
        }

        for (addr, path) in dig_deep {
            self.seek_ab(addr)?;
            self.parse_ifd(path, collector)?;
        }

        Ok(())
    }

    fn parse(&mut self) -> io::Result<Collector> {
        let mut result = HashMap::new();

        // shift to the first entry
        self.seek_re(2)?;
        let ifd_offset = {
            let x = self.read_shift::<4>()?;
            self.u32(x)
        };
        self.seek_ab(ifd_offset)?;

        self.parse_ifd(vec![0], &mut result)?;

        Ok(result)
    }
    fn parse_sony_sr2private(
        &mut self,
        sr2private_index: u16, // the index of sr2private path in path_map
        path: Vec<u16>,
        collector: &mut Collector,
    ) -> io::Result<()> {
        match (
            collector.get(&(sr2private_index, 0x7200)),
            collector.get(&(sr2private_index, 0x7201)),
            collector.get(&(sr2private_index, 0x7221)),
        ) {
            (Some(offset_ifd), Some(length_ifd), Some(key_ifd)) => {
                let offset = self.u32(offset_ifd.value);
                let length = self.u32(length_ifd.value);
                let key = self.u32(key_ifd.value);

                self.seek_ab(offset)?;
                let sr2private_bytes = self.read_to_vec(length as usize)?;
                let decrypted = self.sony_decrypt(&sr2private_bytes, key);
                let mut new_parser = TiffParser {
                    is_le: self.is_le,
                    init_pos: 0,
                    addr_shift: -(offset as i32),
                    reader: BufReader::new(std::io::Cursor::new(decrypted)),
                    path_map: self.path_map.clone(),
                };
                new_parser.parse_ifd(path, collector)?;
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
) -> R<Collector> {
    let reader = BufReader::new(input);

    let mut parser = TiffParser::new(reader, path_dig)?;
    let mut result = parser.parse()?;

    if let Some((sr2private_path_index, sr2private_offset_path_index)) = sony_decrypt_index {
        parser.parse_sony_sr2private(
            sr2private_path_index,
            path_dig[sr2private_offset_path_index].to_vec(),
            &mut result,
        )?;
    }

    Ok(result)
}
