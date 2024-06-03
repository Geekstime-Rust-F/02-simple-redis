use crate::RespDecodeError;
use anyhow::Result;
use bytes::{Buf, BytesMut};

use crate::{parse_length, RespDecode, RespEncode, RespFrame, BUF_CAP, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(Vec<RespFrame>);

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespSet {
    fn encode(self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.0.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode().unwrap());
        }
        Ok(buf)
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespDecode for RespSet {
    const FIRST_BYTE: [u8; 1] = [b'~'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let mut frames = Vec::new();
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;

        buf.advance(length_end_pos + CRLF_LEN);

        for _ in 0..length {
            let value = RespFrame::decode(buf)?;
            frames.push(value);
        }
        Ok(Self::new(frames))
    }
}

impl RespSet {
    pub fn new(frame_vec: impl Into<Vec<RespFrame>>) -> Self {
        Self(frame_vec.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    use crate::resp::{
        bulk_string::RespBulkString, frame::RespFrame, set::RespSet,
        simple_string::RespSimpleString,
    };

    #[test]
    fn test_set_encode() -> Result<()> {
        let frame_vec = vec![RespSimpleString::new("hello").into(), (-1.23456e-8).into()];
        let set = RespSet::new(frame_vec);

        let frame: RespFrame = set.into();
        assert_eq!(
            frame.encode()?,
            b"~2\r\n+hello\r\n,-1.23456e-8\r\n".to_vec()
        );

        Ok(())
    }

    #[test]
    fn test_set_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n+hello\r\n$3\r\nfoo\r\n");
        let frame = RespSet::decode(&mut buf).unwrap();
        let resp_set = RespSet::new(vec![
            RespSimpleString::new("hello".to_string()).into(),
            RespBulkString::new("foo".to_string()).into(),
        ]);
        assert_eq!(frame, resp_set);
    }
}
