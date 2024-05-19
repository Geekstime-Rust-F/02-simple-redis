mod decode;
mod encode;

use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

#[enum_dispatch(RespEncode)]
#[derive(Debug, PartialEq, PartialOrd)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespBulkString(Vec<u8>);
impl RespBulkString {
    pub fn new(string: impl Into<Vec<u8>>) -> Self {
        Self(string.into())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespArray(Vec<RespFrame>);
impl RespArray {
    pub fn new(frame_vec: Vec<RespFrame>) -> Self {
        Self(frame_vec)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<RespSimpleString, RespFrame>);
impl RespMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
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

#[derive(Debug, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);
impl RespSet {
    pub fn new(frame_vec: impl Into<Vec<RespFrame>>) -> Self {
        Self(frame_vec.into())
    }
}

#[enum_dispatch]
trait RespEncode {
    fn encode(self) -> Result<Vec<u8>>;
}

trait RespDecode {
    fn decode(self) -> Result<RespFrame>;
}
