use std::{
    io::{self, BufReader, Read, Seek},
    vec,
};

use crate::ToReport;
use erreport::Report;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Index out of range: {0}")]
    IndexError(usize),
    #[error("Invalid header for JPEG: {0}")]
    InvalidHeader(u16),
    #[error("JPEG tail error: {0:x}")]
    InvalidTail(u16),
}

#[derive(Debug)]
pub struct JPEG<'a> {
    pub dqt: &'a [u8],
    pub sof: SOF,
    pub dht: Vec<DHT<'a>>,
    pub data: &'a [u8],
    pub sos: SOS<'a>,
}

#[derive(Debug)]
pub struct SOF {
    pub id: u8,
    pub precision: u8,
    pub height: u16,
    pub width: u16,
    /// Vec<(component id, horizontal sampling factor, vertical sampling factor, Quantization table destination selector)>
    pub components: Vec<(u8, u8, u8, u8)>,
}

#[derive(Debug)]
pub struct SOS<'a> {
    /// [2bytes for 1 component: Scan component selector + DC entropy coding table destination selector + AC entropy coding table destination selector]
    pub scan_header: Vec<(u8, u8, u8)>,
    /// Start of spectral or predictor selection
    pub ss: u8,
    /// End of spectral selection
    pub se: u8,
    /// Successive approximation bit position high
    pub ah: u8,
    /// Successive approximation bit position low or point transform
    pub al: u8,
    pub body: &'a [u8],
}
#[derive(Debug)]
pub struct DHT<'a> {
    pub is_ac: bool,
    pub destination: u8,
    pub huff_size: &'a [u8; 16],
    pub huff_vals: Vec<&'a [u8]>,
}

// =======================================================================================

impl<'a> Default for JPEG<'a> {
    fn default() -> Self {
        JPEG {
            dqt: &[],
            sof: SOF::default(),
            dht: vec![],
            data: &[],
            sos: SOS::default(),
        }
    }
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
                0xffc0 | 0xffc1 | 0xffc2 | 0xffc3 | 0xffc7 => {
                    let size = bytes.u16(cursor).to_report()? as usize;
                    let sof_id = marker & 0x000f;
                    jpeg.sof = SOF::parse_from_bytes(
                        sof_id as u8,
                        bytes.slice(cursor, size - 2).to_report()?,
                    )
                    .to_report()?;
                }
                0xffc4 => {
                    let size = bytes.u16(cursor).to_report()? as usize;
                    let dhts = DHT::parse_from_bytes(&bytes.slice(cursor, size - 2).to_report()?)
                        .to_report()?;
                    jpeg.dht.extend(dhts);
                }
                0xffda => {
                    let sos = SOS::parse_from_bytes(&bytes[*cursor..])?;
                    jpeg.sos = sos;
                }
                _ => {
                    break;
                }
            }
        }

        Ok(jpeg)
    }
}

impl<'a> Default for SOS<'a> {
    fn default() -> Self {
        SOS {
            scan_header: vec![],
            ss: 0,
            se: 0,
            ah: 0,
            al: 0,
            body: &[],
        }
    }
}
impl Default for SOF {
    fn default() -> Self {
        SOF {
            id: 0,
            precision: 0,
            height: 0,
            width: 0,
            components: vec![],
        }
    }
}
impl SOF {
    fn parse_from_bytes(id: u8, bytes: &[u8]) -> Result<Self, Report> {
        let cursor = &mut 0;
        let precision = bytes.u8(cursor).to_report()?;
        let height = bytes.u16(cursor).to_report()?;
        let width = bytes.u16(cursor).to_report()?;

        let mut components = vec![(0, 0, 0, 0); bytes.u8(cursor).to_report()? as usize];
        for component in components.iter_mut() {
            let c = bytes.u8(cursor).to_report()?;
            let hv = bytes.u8(cursor).to_report()?;
            let h = hv >> 4;
            let v = hv & 0x0f;
            let tq = bytes.u8(cursor).to_report()?;
            *component = (c, h, v, tq);
        }
        Ok(SOF {
            id,
            precision,
            height,
            width,
            components,
        })
    }
}

impl<'a> SOS<'a> {
    fn parse_from_bytes(bytes: &'a [u8]) -> Result<Self, Report> {
        let cursor = &mut 0;
        bytes.u16(cursor).to_report()?; // get scan header length

        let components = bytes.u8(cursor).to_report()?;
        let mut scan_header = Vec::with_capacity(components as usize);
        for _ in 0..components {
            // Scan component selector
            let csj = bytes.u8(cursor).to_report()?;
            let entropy_selector = bytes.u8(cursor).to_report()?;
            // DC entropy coding table destination selector
            let tdj = entropy_selector >> 4;
            // AC entropy coding table destination selector
            let taj = entropy_selector & 0x0f;
            scan_header.push((csj, tdj, taj));
        }

        let ss = bytes.u8(cursor).to_report()?;
        let se = bytes.u8(cursor).to_report()?;

        let successive_approximation = bytes.u8(cursor).to_report()?;
        let ah = successive_approximation >> 4;
        let al = successive_approximation & 0x0f;

        let body = &bytes.slice(cursor, bytes.len() - *cursor - 2).to_report()?;
        let tail = bytes.u16(cursor).to_report()?;

        if tail != 0xffd9 {
            Err(Error::InvalidTail(tail)).to_report()
        } else {
            Ok(SOS {
                scan_header,
                ss,
                se,
                ah,
                al,
                body,
            })
        }
    }
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
        self.get(range)
            .ok_or_else(|| Error::IndexError(*cursor - size))
    }
}
