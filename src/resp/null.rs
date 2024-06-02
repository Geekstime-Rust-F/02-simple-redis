use crate::{RespDecodeError, RespEncode};
use anyhow::Result;
use bytes::{Buf, BytesMut};

use super::decode::RespDecode;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

// - null: "_\r\n"
impl RespEncode for RespNull {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(b"_\r\n".to_vec())
    }
}

// - null: "_\r\n"
impl RespDecode for RespNull {
    const FIRST_BYTE: [u8; 1] = [b'_'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        if buf == "_\r\n" {
            buf.advance(3);
            Ok(Self)
        } else {
            Err(RespDecodeError::InvalidFrame(
                "RespNull requires to start with _".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use bytes::BytesMut;

    use crate::resp::frame::RespFrame;

    use super::*;

    #[test]
    fn test_null_encode() -> Result<()> {
        let resp_null: RespFrame = RespNull.into();
        let result = resp_null.encode()?;
        assert_eq!(result, b"_\r\n");
        Ok(())
    }

    #[test]
    fn test_null_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");
        let frame = RespNull::decode(&mut buf).unwrap();
        assert_eq!(frame, RespNull);
    }
}
