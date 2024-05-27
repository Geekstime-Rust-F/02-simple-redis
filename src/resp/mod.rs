mod decode;
mod encode;

use anyhow::Result;
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(RespSimpleString),
    Error(RespSimpleError),
    BulkError(RespBulkError),
    Integer(RespInteger),
    BulkString(RespBulkString),
    NullBulkString(RespNullBulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[derive(Error, Debug, PartialEq)]
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
    ParseIntError,
}

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Result<Vec<u8>>;
}

pub trait RespFrameFirstByte {
    const FIRST_BYTE: [u8; 1];
}

pub trait RespDecode: Sized {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError>;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RespSimpleString(String);
impl RespSimpleString {
    pub fn new(string: impl Into<String>) -> Self {
        Self(string.into())
    }
}
impl Deref for RespSimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespFrameFirstByte for RespSimpleString {
    const FIRST_BYTE: [u8; 1] = [b'+'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespSimpleError(String);
impl RespSimpleError {
    pub fn new(string: impl Into<String>) -> Self {
        Self(string.into())
    }
}
impl Deref for RespSimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespFrameFirstByte for RespSimpleError {
    const FIRST_BYTE: [u8; 1] = [b'-'];
}
// impl From<RespSimpleString> for RespFrame {
//     fn from(value: RespSimpleString) -> Self {
//         RespFrame::SimpleString(value)
//     }
// }

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespBulkError(Vec<u8>);
impl RespBulkError {
    pub fn new(string: impl Into<Vec<u8>>) -> Self {
        Self(string.into())
    }
}
impl Deref for RespBulkError {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespFrameFirstByte for RespBulkError {
    const FIRST_BYTE: [u8; 1] = [b'!'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespInteger(i64);
impl RespInteger {
    pub fn new(integer: i64) -> Self {
        Self(integer)
    }
}
impl Deref for RespInteger {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl RespFrameFirstByte for RespInteger {
    const FIRST_BYTE: [u8; 1] = [b':'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespBulkString(Vec<u8>);
impl RespBulkString {
    pub fn new(string: impl Into<Vec<u8>>) -> Self {
        Self(string.into())
    }
}
impl RespFrameFirstByte for RespBulkString {
    const FIRST_BYTE: [u8; 1] = [b'$'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;
impl RespFrameFirstByte for RespNullBulkString {
    const FIRST_BYTE: [u8; 1] = [b'$'];
}

impl RespFrameFirstByte for f64 {
    const FIRST_BYTE: [u8; 1] = [b','];
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespArray(Vec<RespFrame>);
impl RespArray {
    pub fn new(frame_vec: Vec<RespFrame>) -> Self {
        Self(frame_vec)
    }
}
impl RespFrameFirstByte for RespArray {
    const FIRST_BYTE: [u8; 1] = [b'*'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;
impl RespFrameFirstByte for RespNullArray {
    const FIRST_BYTE: [u8; 1] = [b'*'];
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNull;
impl RespFrameFirstByte for RespNull {
    const FIRST_BYTE: [u8; 1] = [b'_'];
}

impl RespFrameFirstByte for bool {
    const FIRST_BYTE: [u8; 1] = [b'#'];
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<RespSimpleString, RespFrame>);
impl RespMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}
impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}
impl Deref for RespMap {
    type Target = BTreeMap<RespSimpleString, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl RespFrameFirstByte for RespMap {
    const FIRST_BYTE: [u8; 1] = [b'%'];
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);
impl RespSet {
    pub fn new(frame_vec: impl Into<Vec<RespFrame>>) -> Self {
        Self(frame_vec.into())
    }
}
impl RespFrameFirstByte for RespSet {
    const FIRST_BYTE: [u8; 1] = [b'~'];
}
