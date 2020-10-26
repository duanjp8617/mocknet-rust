use std::fmt;
use std::convert::From;

#[derive(Debug)]
pub enum Kind {
    CapnpError,
}

#[derive(Debug)]
pub struct Error {
    kind: Kind,
    description: String,
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Self {
            kind: Kind::CapnpError,
            description: format!("{}", err),
        }
    }
}

