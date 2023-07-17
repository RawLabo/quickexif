use std::error::Error;

const DIR_LENGTH: usize = env!("CARGO_MANIFEST_DIR").len();

/// You can use `.source()` to get the first real source in Report
pub struct Report {
    file: &'static str,
    line: u32,
    err: Box<dyn Error>,
}
pub(crate) trait ToReport<T> {
    fn to_report(self) -> Result<T, Report>;
}

impl<T, E: std::error::Error + 'static> ToReport<T> for Result<T, E> {
    #[track_caller]
    fn to_report(self) -> Result<T, Report> {
        match self {
            Ok(t) => Ok(t),
            Err(err) => {
                let loc = core::panic::Location::caller();
                Err(Report {
                    file: &loc.file()[DIR_LENGTH + 1..],
                    line: loc.line(),
                    err: err.into(),
                })
            }
        }
    }
}

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, true)
    }
}
impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, false)
    }
}
impl Error for Report {
    /// This method will ignore the report stack and get the first real source
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self.err.downcast_ref::<Report>() {
            Some(e) => e.source(),
            None => Some(self.err.as_ref()),
        }
    }
}

impl Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, is_debug: bool) -> std::fmt::Result {
        write!(
            f,
            "[{}]{}:{} -> {}",
            env!("CARGO_PKG_NAME"),
            self.file,
            self.line,
            if is_debug {
                format!("{:?}", self.err)
            } else {
                self.err.to_string()
            }
        )
    }
}
