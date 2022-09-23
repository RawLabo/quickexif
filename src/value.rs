//! Represents the type of values that may be contained in the EXIF data.
//! 
use super::utility::GetBytesFromInt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The value's type is not '{0}'")]
    ValueTypeIsNotDesired(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    U16(u16),
    U32(u32),
    R64(f64),
    Str(String),
}

impl Value {
    pub(crate) fn size(&self) -> usize {
        match self {
            Value::U16(_) => 2,
            Value::U32(_) => 4,
            Value::Str(_) => 1,
            Value::R64(_) => 8,
        }
    }
}


macro_rules! to_type_value {
    ($t:tt) => {
        pub(crate) fn $t(&self) -> Result<$t, Error> {
            match self {
                &Value::U16(x) => Ok(x as $t),
                &Value::U32(x) => Ok(x as $t),
                &Value::R64(x) => Ok(x as $t),
                _ => Err(Error::ValueTypeIsNotDesired(stringify!($t))),
            }
        }
    };
}

impl Value {
    to_type_value!(u16);
    to_type_value!(u32);
    to_type_value!(i32);
    to_type_value!(f64);
    to_type_value!(usize);

    pub(crate) fn str(&self) -> Result<&str, Error> {
        match self {
            Value::Str(x) => Ok(x.as_str()),
            _ => Err(Error::ValueTypeIsNotDesired("String")),
        }
    }

    pub(crate) fn u8a4(&self, is_le: bool) -> Result<[u8; 4], Error> {
        match self {
            &Value::U32(x) => Ok(x.to_bytes(is_le)),
            _ => Err(Error::ValueTypeIsNotDesired("U32")),
        }
    }
}
