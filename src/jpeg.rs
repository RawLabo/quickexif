use std::{
    io::{self, BufReader, Read, Seek},
    vec,
};

use erreport::Report;

use crate::ToReport;

#[derive(Debug)]
pub struct DHT {
    is_ac: bool,
    destination: u8,
    huff_size: [u8; 16],
    huff_vals: Vec<Vec<u8>>,
}
impl DHT {
    fn parse_from_bytes(bytes: &[u8]) -> Vec<DHT> {
        let mut result = vec![];

        let mut i = 0;
        while i < bytes.len() {
            let tc = bytes[i] >> 4;
            let th = bytes[i] & 0b00001111;
            i += 1;

            let mut huff_size = [0u8; 16];
            huff_size.copy_from_slice(&bytes[i..i + 16]);
            i += 16;

            let mut huff_vals = vec![];
            for &size in huff_size.iter() {
                let size = size as usize;
                let mut huff_val = vec![0u8; size];
                huff_val.copy_from_slice(&bytes[i..i + size]);
                huff_vals.push(huff_val);
                i += size;
            }

            result.push(DHT {
                is_ac: tc == 1,
                destination: th,
                huff_size,
                huff_vals,
            });
        }

        result
    }
}

pub struct JPEG {
    dqt: Box<[u8]>,
    sof: (u8, Box<[u8]>),
    dht: Vec<DHT>,
}
impl Default for JPEG {
    fn default() -> Self {
        JPEG {
            dqt: vec![].into(),
            sof: (0, vec![].into()),
            dht: vec![],
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid header for JPEG: {0:x}")]
    InvalidHeader(u16),
}
impl JPEG {
    pub fn new<I: Read + Seek>(mut input: I) -> Result<Self, Report> {
        let header = input.u16().to_report()?;
        if header != 0xffd8 {
            return Err(Error::InvalidHeader(header)).to_report();
        }

        let mut jpeg = JPEG::default();

        loop {
            let marker = input.u16().to_report()?;
            match marker {
                0xffdb => {
                    let size = input.u16().to_report()? as usize;
                    jpeg.dqt = input.read_to_heap(size - 2).to_report()?;
                }
                0xffc0 | 0xffc1 | 0xffc7 => {
                    let size = input.u16().to_report()? as usize;
                    let sof_id = marker & 0x000f;
                    jpeg.sof = (sof_id as u8, input.read_to_heap(size - 2).to_report()?);
                }
                0xffc4 => {
                    let size = input.u16().to_report()? as usize;
                    let bytes = input.read_to_heap(size - 2).to_report()?;
                    let dhts = DHT::parse_from_bytes(&bytes);
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
    fn u16(&mut self) -> Result<u16, io::Error>;
    fn read_to_heap(&mut self, size: usize) -> Result<Box<[u8]>, io::Error>;
}

impl<I: Read + Seek> Read4JPEG for I {
    fn u16(&mut self) -> Result<u16, io::Error> {
        let mut bytes = [0u8; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }
    fn read_to_heap(&mut self, size: usize) -> Result<Box<[u8]>, io::Error> {
        let mut bytes = vec![0; size];
        self.read_exact(&mut bytes)?;
        Ok(bytes.into_boxed_slice())
    }
}
