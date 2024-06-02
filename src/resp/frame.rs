use enum_dispatch::enum_dispatch;

use crate::{
    RespArray, RespBulkError, RespBulkString, RespInteger, RespMap, RespNull, RespNullArray,
    RespNullBulkString, RespSimpleError, RespSimpleString,
};

use super::set::RespSet;

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
