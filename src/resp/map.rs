use anyhow::Result;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BytesMut};

use crate::{RespDecode, RespDecodeError, RespEncode, RespFrame, RespSimpleString, BUF_CAP};

use super::decode::{parse_length, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(BTreeMap<RespSimpleString, RespFrame>);
impl RespMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.0.encode().unwrap());
            buf.extend_from_slice(&frame.1.encode().unwrap());
        }
        Ok(buf)
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    const FIRST_BYTE: [u8; 1] = [b'%'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let mut frames = Self::new();
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;
        buf.advance(length_end_pos + CRLF_LEN);

        for _ in 0..length {
            let key = RespSimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key, value);
        }
        Ok(frames)
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

#[cfg(test)]
mod tests {
    use crate::RespBulkString;

    use super::*;

    #[test]
    fn test_map_encode() -> Result<()> {
        let mut map = RespMap::new();
        map.insert(
            RespSimpleString::new("hello"),
            RespBulkString::new("world").into(),
        );
        map.insert(RespSimpleString::new("foo"), (-1.23456e-8).into());

        let frame: RespFrame = map.into();
        assert_eq!(
            frame.encode()?,
            b"%2\r\n+foo\r\n,-1.23456e-8\r\n+hello\r\n$5\r\nworld\r\n".to_vec()
        );

        Ok(())
    }

    #[test]
    fn test_map_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespMap::decode(&mut buf).unwrap();
        let mut resp_map = RespMap::new();
        resp_map.insert(
            RespSimpleString::new("hello".to_string()),
            RespBulkString::new(b"world").into(),
        );
        resp_map.insert(
            RespSimpleString::new("foo".to_string()),
            RespBulkString::new(b"bar").into(),
        );
        assert_eq!(frame, resp_map);
    }
}
