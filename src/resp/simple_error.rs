use anyhow::Result;
use bytes::BytesMut;
use std::ops::Deref;

use crate::RespDecodeError;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespSimpleError(String);

// - error: "-Error message\r\n"
impl RespEncode for RespSimpleError {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("-{}\r\n", *self).into())
    }
}

// - error: "-Error message\r\n"
impl RespDecode for RespSimpleError {
    const FIRST_BYTE: [u8; 1] = [b'-'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let content_end_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(content_end_pos + CRLF_LEN);

        Ok(Self::new(String::from_utf8_lossy(
            &data[1..content_end_pos],
        )))
    }
}

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

#[cfg(test)]
mod tests {

    use anyhow::Ok;

    use crate::resp::frame::RespFrame;

    use super::*;

    #[test]
    fn test_error_encode() -> Result<()> {
        let resp_error: RespFrame = RespSimpleError::new("Error").into();
        let result = resp_error.encode()?;
        assert_eq!(result, b"-Error\r\n");
        Ok(())
    }

    #[test]
    fn test_simple_error_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error\r\n");

        let frame: RespSimpleError = RespSimpleError::decode(&mut buf).unwrap();
        assert_eq!(frame, RespSimpleError::new("Error".to_string()));
    }
}
