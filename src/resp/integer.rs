use anyhow::Result;
use bytes::BytesMut;
use std::ops::Deref;

use crate::RespDecodeError;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
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

// - integer: ":[<+|->]<value>\r\n"
impl RespEncode for RespInteger {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(":{}\r\n", self.0).into())
    }
}

// - integer: ":[<+|->]<value>\r\n"
impl RespDecode for RespInteger {
    const FIRST_BYTE: [u8; 1] = [b':'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end_content_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;

        let data = buf.split_to(end_content_pos + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end_content_pos]);
        Ok(RespInteger::new(s.trim().parse()?))
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Ok;
    use bytes::BytesMut;

    use crate::resp::frame::RespFrame;

    use super::*;

    #[test]
    fn test_integer_encode() -> Result<()> {
        let resp_integer: RespFrame = RespInteger::new(1).into();
        let result = resp_integer.encode()?;
        assert_eq!(result, b":1\r\n");

        let resp_integer: RespFrame = RespInteger::new(-1).into();
        let result = resp_integer.encode()?;
        assert_eq!(result, b":-1\r\n");

        Ok(())
    }

    #[test]
    fn test_integer_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":+123\r\n");
        let frame = RespInteger::decode(&mut buf).unwrap();
        assert_eq!(frame, RespInteger::new(123));

        buf.clear();
        buf.extend_from_slice(b":-123\r\n");
        let frame = RespInteger::decode(&mut buf).unwrap();
        assert_eq!(frame, RespInteger::new(-123));
    }
}
