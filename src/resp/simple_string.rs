use anyhow::Result;
use bytes::BytesMut;
use std::ops::Deref;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RespSimpleString(String);

// - simple string: "+OK\r\n"
impl RespEncode for RespSimpleString {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("+{}\r\n", *self).into())
    }
}

// - simple string: "+OK\r\n"
impl RespDecode for RespSimpleString {
    const FIRST_BYTE: [u8; 1] = [b'+'];

    fn decode(buf: &mut BytesMut) -> Result<Self, crate::RespDecodeError> {
        let content_end_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(content_end_pos + CRLF_LEN);

        Ok(Self::new(String::from_utf8_lossy(
            &data[1..content_end_pos],
        )))
    }
}

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

#[cfg(test)]
mod tests {

    use anyhow::Ok;
    use bytes::{BufMut, BytesMut};

    use crate::{resp::frame::RespFrame, RespDecodeError};

    use super::*;

    #[test]
    fn test_simple_string_encode() -> Result<()> {
        let resp_simple_string: RespFrame = RespSimpleString::new("OK").into();
        let result = resp_simple_string.encode()?;
        assert_eq!(result, b"+OK\r\n");
        Ok(())
    }

    #[test]
    fn test_simple_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");
        let frame: RespSimpleString = RespSimpleString::decode(&mut buf).unwrap();
        assert_eq!(frame, RespSimpleString::new("OK".to_string()));

        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r");
        let ret = RespSimpleString::decode(&mut buf).unwrap_err();
        assert_eq!(ret, RespDecodeError::NotComplete);

        buf.put_u8(b'\n');
        let frame = RespSimpleString::decode(&mut buf)?;
        assert_eq!(frame, RespSimpleString::new("OK".to_string()));

        Ok(())
    }
}
