#![macro_use]

use libc::c_int;
use std::error::Error as StdError;
use std::{fmt, str};

/// ALSA error
#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) struct Error(&'static str, i32);

pub(crate) type Result<T> = ::std::result::Result<T, Error>;

macro_rules! acheck {
	($context: expr, $f: ident ( $($x: expr),* ) ) => {{
		let r = unsafe { ($context.$f)( $($x),* ) };
		if r < 0 { Err(Error::new(stringify!($f), -r as ::libc::c_int)) }
		else { Ok(r) }
	}}
}

impl Error {
    pub(crate) fn new(func: &'static str, res: c_int) -> Error {
        Error(func, res as i32)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "ALSA error"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ALSA function '{}' failed with errorno '{}'",
            self.0, self.1
        )
    }
}

impl From<Error> for fmt::Error {
    fn from(_: Error) -> fmt::Error {
        fmt::Error
    }
}
