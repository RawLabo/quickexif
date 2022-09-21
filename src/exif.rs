use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum OffsetType {
    Bytes(isize),
    Address,
    PrevField(&'static str),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CondType {
    LT,
    EQ,
    GT,
    EXIST,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExifTask {
    Tiff(Vec<ExifTask>),

    Condition {
        cond: (CondType, &'static str, u32),
        left: Vec<ExifTask>,
        right: Vec<ExifTask>,
    },
    Offset(OffsetType, Vec<ExifTask>),
    Jump {
        tag: u16,
        is_optional: bool,
        tasks: Vec<ExifTask>,
    },
    JumpNext(Vec<ExifTask>),
    Scan {
        marker: &'static [u8],
        name: Option<&'static str>,
        tasks: Vec<ExifTask>,
    },
    SonyDecrypt {
        offset_tag: u16,
        len_tag: u16,
        key_tag: u16,
        tasks: Vec<ExifTask>,
    },
    TagItem {
        tag: u16,
        name: &'static str,
        len: Option<&'static str>,
        is_optional: bool,
        is_value_u16: bool,
    },
    OffsetItem {
        offset: usize,
        name: &'static str,
        t: Value,
    },
}
