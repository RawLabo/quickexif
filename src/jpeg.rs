use std::{
    io::{self, BufReader, Read, Seek},
    vec,
};

use erreport::Report;

use crate::ToReport;

#[derive(Debug)]
pub struct DHT<'a> {
    is_ac: bool,
    destination: u8,
    huff_size: &'a [u8; 16],
    huff_vals: Vec<&'a [u8]>,
}
impl<'a> DHT<'a> {
    fn parse_from_bytes(bytes: &[u8]) -> Result<Vec<DHT>, Report> {
        let mut result = vec![];

        let cursor = &mut 0;
        while *cursor < bytes.len() {
            let head = bytes.u8(cursor).to_report()?;
            let tc = head >> 4;
            let th = head & 0b00001111;

            let huff_size: &[u8; 16] = bytes
                .slice(cursor, 16)
                .to_report()?
                .try_into()
                .to_report()?;

            let mut huff_vals = vec![];
            for &size in huff_size.iter() {
                let size = size as usize;
                huff_vals.push(bytes.slice(cursor, size).to_report()?);
            }

            result.push(DHT {
                is_ac: tc == 1,
                destination: th,
                huff_size,
                huff_vals,
            });
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub struct JPEG<'a> {
    dqt: &'a [u8],
    sof: (u8, &'a [u8]),
    dht: Vec<DHT<'a>>,
}
impl<'a> Default for JPEG<'a> {
    fn default() -> Self {
        JPEG {
            dqt: &[],
            sof: (0, &[]),
            dht: vec![],
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Index out of range: {0}")]
    IndexError(usize),
    #[error("Invalid header for JPEG: {0}")]
    InvalidHeader(u16),
}

impl<'a> JPEG<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, Report> {
        let cursor = &mut 0;

        let header = bytes.u16(cursor).to_report()?;
        if header != 0xffd8 {
            return Err(Error::InvalidHeader(header)).to_report();
        }

        let mut jpeg = JPEG::default();
        loop {
            let marker = bytes.u16(cursor).to_report()?;
            match marker {
                0xffdb => {
                    let size = bytes.u16(cursor).to_report()? as usize;
                    jpeg.dqt = bytes.slice(cursor, size - 2).to_report()?;
                }
                0xffc0 | 0xffc1 | 0xffc7 => {
                    let size = bytes.u16(cursor).to_report()? as usize;
                    let sof_id = marker & 0x000f;
                    jpeg.sof = (sof_id as u8, bytes.slice(cursor, size - 2).to_report()?);
                }
                0xffc4 => {
                    let size = bytes.u16(cursor).to_report()? as usize;
                    let dhts = DHT::parse_from_bytes(&bytes.slice(cursor, size - 2).to_report()?)
                        .to_report()?;
                    jpeg.dht.extend(dhts);
                }
                _ => {
                    break;
                }
            }
        }

        Ok(jpeg)
    }
}

trait Read4JPEG {
    fn u8(&self, cursor: &mut usize) -> Result<u8, Error>;
    fn u16(&self, cursor: &mut usize) -> Result<u16, Error>;
    fn slice<'a>(&'a self, cursor: &mut usize, size: usize) -> Result<&'a [u8], Error>;
}

impl Read4JPEG for [u8] {
    fn u8(&self, cursor: &mut usize) -> Result<u8, Error> {
        let data = self
            .get(*cursor)
            .ok_or_else(|| Error::IndexError(*cursor))?;
        *cursor += 1;
        Ok(*data)
    }
    fn u16(&self, cursor: &mut usize) -> Result<u16, Error> {
        let mut x = [0u8; 2];
        let data = self
            .get(*cursor..*cursor + 2)
            .ok_or_else(|| Error::IndexError(*cursor))?;
        x.copy_from_slice(&data);
        *cursor += 2;
        Ok(u16::from_be_bytes(x))
    }
    fn slice<'a>(&'a self, cursor: &mut usize, size: usize) -> Result<&'a [u8], Error> {
        let range = *cursor..*cursor + size;
        *cursor += size;
        self.get(range).ok_or_else(|| Error::IndexError(*cursor - size))
    }
}
