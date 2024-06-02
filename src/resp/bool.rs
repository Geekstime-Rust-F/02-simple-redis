use crate::RespDecodeError;
use anyhow::Result;
use bytes::{Buf, BytesMut};

use crate::{extract_simple_frame_data, RespDecode, RespEncode};

// - boolean: "#<t|f>\r\n"
impl RespEncode for bool {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!("#{}\r\n", if self { "t" } else { "f" }).into())
    }
}

// - boolean: "#<t|f>\r\n"
impl RespDecode for bool {
    const FIRST_BYTE: [u8; 1] = [b'#'];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end_content_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let s = String::from_utf8_lossy(&buf[1..end_content_pos]);
        match s.trim() {
            "t" => {
                buf.advance(4);
                Ok(true)
            }
            "f" => {
                buf.advance(4);
                Ok(false)
            }
            _ => Err(RespDecodeError::InvalidFrame(
                "RespBoolean requires to be t or f".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use anyhow::{Ok, Result};
    use bytes::BytesMut;

    use crate::resp::decode::RespDecode;
    use crate::resp::frame::RespFrame;
    use crate::resp::RespEncode;

    #[test]
    fn test_bool_true_encode() -> Result<()> {
        let resp_bool: RespFrame = true.into();
        let result = resp_bool.encode()?;
        assert_eq!(result, b"#t\r\n");

        let resp_bool: RespFrame = false.into();
        let result = resp_bool.encode()?;
        assert_eq!(result, b"#f\r\n");

        Ok(())
    }

    #[test]
    fn test_boolean_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"#t\r\n");
        let frame = bool::decode(&mut buf).unwrap();
        assert!(frame);

        buf.clear();
        buf.extend_from_slice(b"#f\r\n");
        let frame = bool::decode(&mut buf).unwrap();
        assert!(!frame);
    }
}
