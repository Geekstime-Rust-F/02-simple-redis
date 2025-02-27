use anyhow::Result;
use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::RespDecodeError;

use crate::{parse_length, RespDecode, RespEncode, CRLF, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespBulkString(pub Vec<u8>);
impl RespBulkString {
    pub fn new(string: impl Into<Vec<u8>>) -> Self {
        Self(string.into())
    }
}
impl Deref for RespBulkString {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<[u8]> for RespBulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<String> for RespBulkString {
    fn from(value: String) -> Self {
        RespBulkString(value.into_bytes())
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for RespBulkString {
    fn encode(self) -> Result<Vec<u8>> {
        if self.0.is_empty() {
            return Ok(b"$-1\r\n".to_vec());
        }
        Ok(format!(
            "${}\r\n{}\r\n",
            self.0.len(),
            String::from_utf8(self.0).unwrap()
        )
        .into())
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
// - null bulk string: "$-1\r\n"
impl RespDecode for RespBulkString {
    const FIRST_BYTE: [u8; 1] = [b'$'];

    fn decode(buf: &mut BytesMut) -> std::result::Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;

        if length == -1 {
            buf.advance(5);
            return Ok(Self::new(Vec::new()));
        }
        let length: usize = length as usize;

        buf.advance(length_end_pos + CRLF_LEN);
        let bulk_string = buf.split_to(length + CRLF_LEN);
        if &bulk_string[length..] == CRLF.as_bytes() {
            Ok(RespBulkString::new(&bulk_string[0..length]))
        } else {
            Err(RespDecodeError::InvalidFrame(format!(
                "RespBulkString didn't end with {} or length not match",
                CRLF
            )))
        }
    }
}

#[cfg(test)]
mod tests {

    use bytes::BytesMut;

    use crate::{resp::frame::RespFrame, RespDecodeError};

    use super::*;

    #[test]
    fn test_bulk_string_encode() -> Result<()> {
        let resp_bulk_string: RespFrame = RespBulkString::new("hello").into();
        let result = resp_bulk_string.encode()?;
        assert_eq!(result, b"$5\r\nhello\r\n");
        Ok(())
    }

    #[test]
    fn test_null_bulk_string_encode() -> Result<()> {
        let resp_null_bulk_string: RespFrame = RespBulkString::new("").into();
        let result = resp_null_bulk_string.encode()?;
        assert_eq!(result, b"$-1\r\n");
        Ok(())
    }

    #[test]
    fn test_bulk_string_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$13\r\nstring string\r\n");
        let frame = RespBulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, RespBulkString::new("string string".to_string()));

        buf.clear();
        buf.extend_from_slice(b"$13\r\nstring string\r\n\r\n");
        let frame = RespBulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, RespBulkString::new("string string".to_string()));

        buf.clear();
        buf.extend_from_slice(b"$13\r\nstring stringx\r\n");
        let result = RespBulkString::decode(&mut buf).unwrap_err();
        assert_eq!(
            result,
            RespDecodeError::InvalidFrame(
                "RespBulkString didn't end with \r\n or length not match".to_string()
            )
        );
    }

    #[test]
    fn test_null_bulk_string_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");
        let frame = RespBulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, RespBulkString::new(Vec::new()));
    }
}
