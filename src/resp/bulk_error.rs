use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

use crate::RespDecodeError;

use crate::{parse_length, RespDecode, RespEncode, CRLF, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
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

// - bulk error: "!<length>\r\n<error>\r\n"
impl RespEncode for RespBulkError {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(
            "!{}\r\n{}\r\n",
            self.0.len(),
            String::from_utf8(self.0).unwrap()
        )
        .into())
    }
}

// - bulk error: "!<length>\r\n<error>\r\n"
impl RespDecode for RespBulkError {
    const FIRST_BYTE: [u8; 1] = [b'!'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;
        if length == -1 {
            buf.advance(5);
            return Ok(Self::new(Vec::new()));
        }
        let length: usize = length as usize;
        buf.advance(length_end_pos + CRLF_LEN);
        let error = buf.split_to(length + CRLF_LEN);
        if &error[length..] == CRLF.as_bytes() {
            Ok(RespBulkError::new(&error[0..length]))
        } else {
            Err(RespDecodeError::InvalidFrame(format!(
                "RespBulkError didn't end with {} or length not match",
                CRLF
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    use crate::{
        resp::{bulk_error::RespBulkError, frame::RespFrame},
        RespDecodeError,
    };

    #[test]
    fn test_bulk_error_encode() -> Result<()> {
        let resp_bulk_error: RespFrame = RespBulkError::new("Error").into();
        let result = resp_bulk_error.encode()?;
        assert_eq!(result, b"!5\r\nError\r\n");
        Ok(())
    }

    #[test]
    fn test_bulk_error_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"!11\r\nerror error\r\n");
        let frame = RespBulkError::decode(&mut buf).unwrap();
        assert_eq!(frame, RespBulkError::new("error error".to_string()));

        buf.clear();
        buf.extend_from_slice(b"!11\r\nerror error\r\n\r\n");
        let frame = RespBulkError::decode(&mut buf).unwrap();
        assert_eq!(frame, RespBulkError::new("error error".to_string()));

        buf.clear();
        buf.extend_from_slice(b"!11\r\nerror errorx\r\n");
        let result = RespFrame::decode(&mut buf).unwrap_err();
        assert_eq!(
            result,
            RespDecodeError::InvalidFrame(
                "RespBulkError didn't end with \r\n or length not match".to_string()
            )
        );
    }
}
