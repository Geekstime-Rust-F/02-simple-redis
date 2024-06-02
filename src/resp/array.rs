use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

use crate::RespDecodeError;

use crate::{parse_length, RespDecode, RespEncode, RespFrame, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub Vec<RespFrame>);
impl RespArray {
    pub fn new(frame_vec: Vec<RespFrame>) -> Self {
        Self(frame_vec)
    }
}
impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//   - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
const ARRAY_CAP: usize = 4096;
impl RespEncode for RespArray {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(ARRAY_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());

        for frame in self.0 {
            buf.extend_from_slice(&frame.encode().unwrap());
        }

        Ok(buf)
    }
}

// - null array: "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(b"*-1\r\n".to_vec())
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//    - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// - null array: "*-1\r\n"
impl RespDecode for RespArray {
    const FIRST_BYTE: [u8; 1] = [b'*'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;
        buf.advance(length_end_pos + CRLF_LEN);

        let mut frames = Vec::new();
        for _ in 0..length {
            let value = RespFrame::decode(buf)?;
            frames.push(value);
        }
        Ok(Self::new(frames))
    }
}

impl RespDecode for RespNullArray {
    const FIRST_BYTE: [u8; 1] = [b'*'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        if buf == "*-1\r\n" {
            buf.advance(5);
            Ok(Self)
        } else {
            Err(RespDecodeError::InvalidFrame(
                "RespNullArray requires to start with $".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Ok;
    use bytes::BytesMut;

    use crate::resp::{
        bulk_string::{RespBulkString, RespNullBulkString},
        simple_string::RespSimpleString,
    };

    use super::*;

    #[test]
    fn test_array_encode() -> Result<()> {
        let frame_vec = vec![
            RespNullBulkString.into(),
            RespBulkString::new("hello").into(),
        ];
        let resp_array = RespArray::new(frame_vec);
        let result = resp_array.encode()?;
        assert_eq!(result, b"*2\r\n$-1\r\n$5\r\nhello\r\n");
        Ok(())
    }

    #[test]
    fn test_null_array_encode() -> Result<()> {
        let resp_null_array: RespFrame = RespNullArray.into();
        let result = resp_null_array.encode()?;
        assert_eq!(result, b"*-1\r\n");
        Ok(())
    }

    #[test]
    fn test_array_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespArray::new(vec![
                RespBulkString::new(b"hello").into(),
                RespBulkString::new(b"world").into()
            ])
        );

        buf.clear();
        buf.extend_from_slice(b"*2\r\n$5\r\nhello\r\n+OK\r\n");
        let frame = RespArray::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespArray::new(vec![
                RespBulkString::new(b"hello").into(),
                RespSimpleString::new("OK".to_string()).into()
            ])
        );
    }

    #[test]
    fn test_null_array_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");
        let frame = RespNullArray::decode(&mut buf).unwrap();
        assert_eq!(frame, RespNullArray);
    }
}
