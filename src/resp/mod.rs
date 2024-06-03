mod array;
mod bool;
mod bulk_error;
mod bulk_string;
mod decode;
mod f64;
mod frame;
mod integer;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

pub use self::{
    array::RespArray,
    bulk_error::RespBulkError,
    bulk_string::RespBulkString,
    decode::{extract_simple_frame_data, parse_length, RespDecode, CRLF, CRLF_LEN},
    frame::RespFrame,
    integer::RespInteger,
    map::RespMap,
    null::RespNull,
    set::RespSet,
    simple_error::RespSimpleError,
    simple_string::RespSimpleString,
};

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RespDecodeError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),

    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(usize),

    #[error("Frame is not complete")]
    NotComplete,

    #[error("Frame parse int error")]
    ParseIntError(#[from] ParseIntError),
    // ParseIntError,
    #[error("Frame parse float error")]
    ParseFloatError(#[from] ParseFloatError),
}

pub const BUF_CAP: usize = 1024;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Result<Vec<u8>>;
}

// implementation of Redis serialization protocol
/*
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - big number: "([+|-]<number>\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
*/
