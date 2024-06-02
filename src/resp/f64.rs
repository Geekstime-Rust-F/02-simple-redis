use crate::RespDecodeError;
use anyhow::Result;
use bytes::BytesMut;

use crate::{extract_simple_frame_data, RespDecode, RespEncode, CRLF_LEN};

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespEncode for f64 {
    fn encode(self) -> Result<Vec<u8>> {
        Ok(format!(",{:+e}\r\n", self).into())
    }
}

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespDecode for f64 {
    const FIRST_BYTE: [u8; 1] = [b','];

    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end_content_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(end_content_pos + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end_content_pos]);
        Ok(s.trim().parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bytes::BytesMut;

    use crate::resp::frame::RespFrame;

    #[test]
    fn test_double_encode() -> Result<()> {
        let resp_double: RespFrame = 123.4567.into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",+1.234567e2\r\n");

        let resp_double: RespFrame = (-1.0).into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",-1e0\r\n");

        let resp_double: RespFrame = 1.23456e+8.into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",+1.23456e8\r\n");

        let resp_double: RespFrame = (-1.23456e-8).into();
        let result = resp_double.encode()?;
        assert_eq!(result, b",-1.23456e-8\r\n");

        Ok(())
    }

    #[test]
    fn test_double_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",123.456\r\n");
        let frame = f64::decode(&mut buf).unwrap();
        assert_eq!(frame, 123.456);

        buf.clear();
        buf.extend_from_slice(b",-1.23456e-9\r\n");
        let frame = f64::decode(&mut buf).unwrap();
        assert_eq!(frame, -1.23456e-9);
    }
}
