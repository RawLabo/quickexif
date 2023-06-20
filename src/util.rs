pub(crate) type R<T> = Result<T, Box<dyn std::error::Error>>;

macro_rules! log_err {
    ($($body:tt)*) => {
        {
            match (|| -> R<_> {
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
pub(crate) use log_err;

pub(crate) mod opt_none {
    #[derive(thiserror::Error, Debug)]
    pub(crate) enum OptNone {
        #[error("Option {0} is None")]
        Name(&'static str),
    }
    
    macro_rules! opt_none {
        ($name:ident) => {
            pub(crate) fn $name() -> OptNone {
                OptNone::Name(stringify!($name))
            }
        };
    }

}