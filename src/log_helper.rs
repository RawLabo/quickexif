use std::error::Error;
use std::fmt::{Debug, Display};

pub type LogResult<T> = Result<T, ErrLog>;

/// You can use `.source()` to get the first real source in ErrLog 
pub struct ErrLog {
    file: &'static str,
    line: u32,
    err: Box<dyn std::error::Error>,
}
impl ErrLog {
    pub fn new(file: &'static str, line: u32, err: Box<dyn std::error::Error>) -> Self {
        ErrLog { file, line, err }
    }
}
impl Debug for ErrLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} -> {:?}", self.file, self.line, self.err)
    }
}
impl Display for ErrLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl Error for ErrLog {
    /// This method will ignore the stack and get the first real source in ErrLog
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.err.downcast_ref::<ErrLog>() {
            Some(e) => e.source(),
            None => Some(self.err.as_ref())
        }
    }
}

#[macro_export]
macro_rules! log_err {
    ($err:expr) => {
        ErrLog::new(file!(), line!(), $err.into())
    };
}
pub use log_err;

#[macro_export]
macro_rules! q {
    ($result:expr) => {
        $result.map_err(|err| log_err!(err))?
    };
}
pub use q;

#[macro_export]
macro_rules! with_result {
    ($($body:tt)*) => {
        {
            match (|| -> LogResult<_> {
                $($body)*
            })() {
                Err(err) => {
                    log::error!("{:#?}", err);
                    panic!("{:#?}", err);
                },
                Ok(result) => result
            }
        }
    }
}
pub use with_result;