use super::*;
use exif::*;
use parsed_info::*;
use std::collections::HashMap;
use utility::GetNumFromBytes;
use value::*;

use thiserror::Error;

const TIFF_LITTLE_ENDIAN: u16 = 0x4949;
const TIFF_BIG_ENDIAN: u16 = 0x4d4d;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parsing exif error.")]
    RawInfoError(#[from] parsed_info::Error),
    #[error("The byte order of tiff header {0:#02x?} is invalid.")]
    InvalidTiffHeaderByteOrder(u16),
    #[error("The tag {0:#02x?} was not found.")]
    TagNotFound(u16),
    #[error("The start task should be Tiff or JPEG.")]
    InvalidStartTask,
    #[error("Scan failed to find '{0:#02x?}'.")]
    ScanFailed(&'static [u8]),
}

pub struct Parser<'a> {
    is_le: bool, // is little endian
    buffer: &'a [u8],
    offset: usize,
    entries: HashMap<u16, &'a [u8]>,
    next_offset: usize,
}

impl<'a> Parser<'a> {
    fn is_byte_order_le(buffer: &[u8], start: usize) -> Result<bool, Error> {
        let byte_order = buffer.u16(true, start);
        match byte_order {
            TIFF_LITTLE_ENDIAN => Ok(true),
            TIFF_BIG_ENDIAN => Ok(false),
            _ => Err(Error::InvalidTiffHeaderByteOrder(byte_order)),
        }
    }

    pub fn parse(buffer: &[u8], task: &exif::ExifTask) -> Result<ParsedInfo, Error> {
        let content = HashMap::new();
        Self::get_info_with_content(buffer, task, content)
    }

    fn get_info_with_content(
        buffer: &[u8],
        task: &exif::ExifTask,
        mut content: HashMap<String, Value>,
    ) -> Result<ParsedInfo, Error> {
        let buffer = if buffer[..2] == [0xff, 0xd8] {
            // JPEG header fix
            &buffer[12..]
        } else {
            buffer
        };

        let decoder = Parser {
            is_le: Parser::is_byte_order_le(buffer, 0)?,
            buffer,
            offset: 0,
            entries: HashMap::new(),
            next_offset: 0,
        };
        decoder.run_task_body(task, &mut content)?;
        Ok(ParsedInfo {
            is_le: decoder.is_le,
            content,
        })
    }

    fn is_tasks_in_ifd(tasks: &[exif::ExifTask]) -> bool {
        tasks.iter().any(|x| {
            matches!(
                x,
                exif::ExifTask::Jump {
                    tag: _,
                    is_optional: _,
                    tasks: _
                } | exif::ExifTask::TagItem {
                    tag: _,
                    name: _,
                    len: _,
                    is_optional: _,
                    is_value_u16: _,
                } | exif::ExifTask::SonyDecrypt {
                    offset_tag: _,
                    len_tag: _,
                    key_tag: _,
                    tasks: _
                }
            )
        })
    }
    fn read_u32value_from_entries(
        &self,
        tag: u16,
        custom_offset: Option<usize>,
    ) -> Result<u32, Error> {
        let tag_line = self.entries.get(&tag).ok_or(Error::TagNotFound(tag))?;
        Ok(tag_line.u32(self.is_le, custom_offset.unwrap_or(8)))
    }
    fn read_u16value_from_entries(
        &self,
        tag: u16,
        custom_offset: Option<usize>,
    ) -> Result<u16, Error> {
        let tag_line = self.entries.get(&tag).ok_or(Error::TagNotFound(tag))?;
        Ok(tag_line.u16(self.is_le, custom_offset.unwrap_or(8)))
    }
    fn read_value_from_offset(&self, offset: usize, t: &Value) -> Value {
        let offset = self.offset + offset * t.size();
        match t {
            Value::U16(_) => Value::U16(self.buffer.u16(self.is_le, offset)),
            Value::U32(_) => Value::U32(self.buffer.u32(self.is_le, offset)),
            Value::Str(_) => {
                let str: String = self.buffer[offset..]
                    .iter()
                    .map_while(|&x| if x == 0 { None } else { Some(x as char) })
                    .collect();
                Value::Str(str.trim().to_owned())
            }
            Value::R64(_) => Value::R64(self.buffer.r64(self.is_le, offset)),
        }
    }

    fn run_remain_tasks(
        &mut self,
        tasks: &[exif::ExifTask],
        content: &mut HashMap<String, Value>,
    ) -> Result<(), Error> {
        // IFD entry check
        if Parser::is_tasks_in_ifd(tasks) {
            let entry_count = self.buffer.u16(self.is_le, self.offset) as usize;
            self.offset += 2;

            for tag_line in self.buffer[self.offset..]
                .chunks_exact(12)
                .take(entry_count)
            {
                let tag = tag_line.u16(self.is_le, 0);
                self.entries.insert(tag, tag_line);
            }

            self.next_offset = self.buffer.u32(self.is_le, self.offset + entry_count * 12) as usize;
        }

        for task in tasks.iter() {
            self.run_task_body(task, content)?;
        }
        Ok(())
    }
    fn run_task_body(
        &self,
        task: &exif::ExifTask,
        content: &mut HashMap<String, Value>,
    ) -> Result<(), Error> {
        match task {
            // blocks
            exif::ExifTask::Tiff(tasks) => {
                let is_le = if self.offset == 0 {
                    self.is_le
                } else {
                    Parser::is_byte_order_le(self.buffer, self.offset)?
                };

                let new_buffer = &self.buffer[self.offset..];
                let mut new_parser = Parser {
                    is_le,
                    buffer: new_buffer,
                    offset: new_buffer.u32(is_le, 4) as usize,
                    entries: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &exif::ExifTask::Condition {
                cond,
                ref left,
                ref right,
            } => {
                let (cond_type, field, target) = cond;
                let result = match cond_type {
                    CondType::LT | CondType::EQ | CondType::GT => {
                        let value = content
                            .get(field)
                            .ok_or_else(|| parsed_info::Error::FieldNotFound(field.to_owned()))?
                            .u32()
                            .map_err(parsed_info::Error::InvalidValue)?;

                        match cond_type {
                            CondType::LT => value < target,
                            CondType::EQ => value == target,
                            CondType::GT => value > target,
                            _ => value == target,
                        }
                    }
                    CondType::EXIST => content.get(field).is_some(),
                };

                for task in if result { left } else { right }.iter() {
                    self.run_task_body(task, content)?;
                }
            }
            &exif::ExifTask::JumpNext(ref tasks) => {
                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: self.buffer,
                    offset: self.next_offset,
                    entries: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &exif::ExifTask::Jump {
                tag,
                is_optional,
                ref tasks,
            } => {
                let offset = self.read_u32value_from_entries(tag, None);
                match (offset, is_optional) {
                    (Ok(offset), _) => {
                        let mut new_parser = Parser {
                            is_le: self.is_le,
                            buffer: self.buffer,
                            offset: offset as usize,
                            entries: HashMap::new(),
                            next_offset: 0,
                        };
                        new_parser.run_remain_tasks(tasks, content)?;
                    }
                    (Err(e), false) => Err(e)?,
                    _ => {}
                }
            }
            &exif::ExifTask::Scan {
                marker,
                name,
                ref tasks,
            } => {
                let &(offset, _) = &self.buffer[self.offset..]
                    .windows(marker.len())
                    .enumerate()
                    .find(|(_, data)| data == &marker)
                    .ok_or(Error::ScanFailed(marker))?;

                let tiff_offset = offset + self.offset;
                if let Some(n) = name {
                    content.insert(n.to_owned(), Value::U32(tiff_offset as u32));
                }

                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: &self.buffer[tiff_offset..],
                    offset: 0,
                    entries: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &exif::ExifTask::Offset(ref offset, ref tasks) => {
                let new_offset = match offset {
                    OffsetType::Bytes(0) => {
                        for task in tasks.iter() {
                            self.run_task_body(task, content)?;
                        }
                        return Ok(());
                    }
                    OffsetType::Bytes(x) => (self.offset as isize + x) as usize,
                    OffsetType::Address => {
                        (&self.buffer[self.offset..]).u32(self.is_le, 0) as usize
                    }
                    &OffsetType::PrevField(field) => {
                        self.offset
                            + content
                                .get(field)
                                .ok_or_else(|| parsed_info::Error::FieldNotFound(field.to_owned()))?
                                .usize()
                                .map_err(parsed_info::Error::InvalidValue)?
                    }
                };
                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: self.buffer,
                    offset: new_offset,
                    entries: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            &exif::ExifTask::SonyDecrypt {
                offset_tag,
                len_tag,
                key_tag,
                ref tasks,
            } => {
                let offset = self.read_u32value_from_entries(offset_tag, None)? as usize;
                let len = self.read_u32value_from_entries(len_tag, None)? as usize;
                let key = self.read_u32value_from_entries(key_tag, None)?;
                let mut decrypted = vec![0u8; offset];
                decrypted.append(&mut utility::sony_decrypt(
                    &self.buffer[offset..offset + len],
                    key,
                    self.is_le,
                ));

                let mut new_parser = Parser {
                    is_le: self.is_le,
                    buffer: &decrypted,
                    offset,
                    entries: HashMap::new(),
                    next_offset: 0,
                };
                new_parser.run_remain_tasks(tasks, content)?;
            }
            // items
            &exif::ExifTask::TagItem {
                tag,
                name,
                len,
                is_optional,
                is_value_u16,
            } => {
                let value = if is_value_u16 {
                    self.read_u16value_from_entries(tag, None).map(Value::U16)
                } else {
                    self.read_u32value_from_entries(tag, None).map(Value::U32)
                };
                match (value, is_optional) {
                    (Ok(v), _) => {
                        content.insert(name.to_owned(), v);
                        if let Some(len_name) = len {
                            let value = self.read_u32value_from_entries(tag, Some(4))?;
                            content.insert(len_name.to_owned(), Value::U32(value));
                        }
                    }
                    (Err(e), false) => Err(e)?,
                    _ => {}
                };
            }
            &exif::ExifTask::OffsetItem {
                offset,
                name,
                ref t,
            } => {
                let value = self.read_value_from_offset(offset, t);
                content.insert(name.to_owned(), value);
            }
        };
        Ok(())
    }
}
