use super::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The raw has invalid info value.")]
    InvalidValue(#[from] value::Error),
    #[error("The collector from decoder does not contain the '{0}' field.")]
    FieldNotFound(String),
    #[error("The value's type of the field:'{0}' is invalid")]
    FieldValueIsInvalid(String),
}

pub struct ParsedInfo {
    pub is_le: bool,
    pub(crate) content: HashMap<&'static str, value::Value>,
}

macro_rules! gen_collector_impls_for_num {
    ($t:tt) => {
        pub fn $t(&self, name: &str) -> Result<$t, Error> {
            match self.content.get(name) {
                Some(v) => Ok(v.$t()?),
                None => Err(Error::FieldNotFound(name.to_owned())),
            }
        }
    };
}

impl ParsedInfo {
    gen_collector_impls_for_num!(u16);
    gen_collector_impls_for_num!(u32);
    gen_collector_impls_for_num!(i32);
    gen_collector_impls_for_num!(f64);
    gen_collector_impls_for_num!(usize);

    pub fn str<'a>(&'a self, name: &str) -> Result<&'a str, Error> {
        match self.content.get(name) {
            Some(v) => Ok(v.str()?),
            None => Err(Error::FieldNotFound(name.to_owned())),
        }
    }

    pub fn u8a4(&self, name: &str) -> Result<[u8; 4], Error> {
        match self.content.get(name) {
            Some(v) => Ok(v.u8a4(self.is_le)?),
            None => Err(Error::FieldNotFound(name.to_owned())),
        }
    }

    pub fn stringify_all(&self) -> Result<String, value::Error> {
        let mut result = format!(
            "{:>22}:  {}-endian\n",
            "endianness",
            if self.is_le { "little" } else { "big" }
        );
        let mut names = self.content.iter().collect::<Vec<_>>();
        names.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, value) in names.iter() {
            let value_str = match value {
                value::Value::U16(x) => format!("{} / {:#x?}", x, x),
                value::Value::U32(x) => {
                    format!("{} / {:#x?} / {:?}", x, x, value.u8a4(self.is_le)?)
                }
                value::Value::R64(x) => x.to_string(),
                value::Value::Str(_) => value.str()?.to_owned(),
            };
            result.push_str(format!("{:>22}:  {}\n", name, value_str).as_str());
        }

        Ok(result)
    }
}
