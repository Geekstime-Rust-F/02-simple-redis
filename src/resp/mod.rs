mod decode;
mod encode;

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::ops::Deref;

#[enum_dispatch(RespEncode)]
pub enum RespFrame {
    SimpleString(RespSimpleString),
    Error(RespError),
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

pub struct RespError(String);
impl RespError {
    pub fn new(string: impl Into<String>) -> Self {
        Self(string.into())
    }
}
impl Deref for RespError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

pub struct RespBulkString(Vec<u8>);
impl RespBulkString {
    pub fn new(string: impl Into<Vec<u8>>) -> Self {
        Self(string.into())
    }
}

pub struct RespNullBulkString;

pub struct RespArray(Vec<RespFrame>);
impl RespArray {
    pub fn new(frame_vec: Vec<RespFrame>) -> Self {
        Self(frame_vec)
    }
}

pub struct RespNullArray;

pub struct RespNull;

pub struct RespMap(Vec<(RespFrame, RespFrame)>);
impl RespMap {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

pub struct RespSet(Vec<RespFrame>);
impl RespSet {
    pub fn new(frame_vec: Vec<RespFrame>) -> Self {
        Self(frame_vec)
    }
}

#[enum_dispatch]
trait RespEncode {
    fn encode(self) -> Result<Vec<u8>>;
}

trait RespDecode {
    fn decode(self) -> Result<RespFrame>;
}
