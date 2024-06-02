// implementation of Redis serialization protocol
/*
    - simple string: "+OK\r\n"
    - error: "-Error message\r\n"
    - bulk error: "!<length>\r\n<error>\r\n"
    - integer: ":[<+|->]<value>\r\n"
    - bulk string: "$<length>\r\n<data>\r\n"
    - null bulk string: "$-1\r\n"
    - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
        - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
    - null array: "*-1\r\n"
    - null: "_\r\n"
    - boolean: "#<t|f>\r\n"
    - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
    - big number: "([+|-]<number>\r\n"
    - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
    - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
*/

use crate::{RespFrameFirstByte, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet};
use anyhow::Result;
use bytes::{Buf, BytesMut};
use tracing::info;

use crate::{
    RespArray, RespBulkError, RespBulkString, RespDecode, RespDecodeError, RespFrame, RespInteger,
    RespSimpleError, RespSimpleString,
};

const CRLF_LEN: usize = 2;
const CRLF: &str = "\r\n";

impl RespDecode for RespFrame {
    fn decode(buf: &mut BytesMut) -> Result<Self, super::RespDecodeError> {
        if buf.len() < 3 {
            return Err(crate::RespDecodeError::NotComplete);
        }
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => Ok(RespSimpleString::decode(buf)?.into()),
            Some(b'-') => Ok(RespSimpleError::decode(buf)?.into()),
            Some(b'!') => Ok(RespBulkError::decode(buf)?.into()),
            Some(b':') => Ok(RespInteger::decode(buf)?.into()),
            Some(b'$') => {
                // try null bulk string first
                match RespNullBulkString::decode(buf) {
                    Ok(frame) => Ok(frame.into()),
                    Err(RespDecodeError::NotComplete) => Err(RespDecodeError::NotComplete),
                    Err(_) => {
                        let frame = RespBulkString::decode(buf)?;
                        Ok(frame.into())
                    }
                }
            }
            Some(b'*') => match RespNullArray::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespDecodeError::NotComplete) => Err(RespDecodeError::NotComplete),
                Err(_) => {
                    let frame = RespArray::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'%') => Ok(RespMap::decode(buf)?.into()),
            Some(b'~') => Ok(RespSet::decode(buf)?.into()),
            Some(b'_') => Ok(RespNull::decode(buf)?.into()),
            Some(b'#') => Ok(bool::decode(buf)?.into()),
            Some(b',') => Ok(f64::decode(buf)?.into()),
            None => Err(RespDecodeError::NotComplete),
            _ => Err(RespDecodeError::InvalidFrame("Invalid frame".to_string())),
        }
    }
}

fn find_nth_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 0..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

fn extract_simple_frame_data(
    buf: &mut BytesMut,
    prefix: [u8; 1],
) -> Result<usize, RespDecodeError> {
    info!("buf in extract_simple_frame_data: {:?}", buf);
    if !buf.starts_with(&prefix) {
        return Err(RespDecodeError::InvalidFrameType(format!(
            "This RespFrame requires to start with {:?}",
            String::from_utf8_lossy(prefix.as_ref())
        )));
    }

    match find_nth_crlf(buf, 1) {
        Some(pos) => Ok(pos),
        None => Err(RespDecodeError::NotComplete),
    }
}

fn parse_length(buf: &mut BytesMut, prefix: &str) -> Result<(usize, usize), RespDecodeError> {
    let length_end_pos = extract_simple_frame_data(buf, [prefix.as_bytes()[0]])?;
    let length = String::from_utf8_lossy(&buf[prefix.len()..length_end_pos]);
    Ok((length_end_pos, length.parse()?))
}

// - simple string: "+OK\r\n"
impl RespDecode for RespSimpleString {
    fn decode(buf: &mut BytesMut) -> Result<Self, crate::RespDecodeError> {
        let content_end_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(content_end_pos + CRLF_LEN);

        Ok(Self::new(String::from_utf8_lossy(
            &data[1..content_end_pos],
        )))
    }
}

// - error: "-Error message\r\n"
impl RespDecode for RespSimpleError {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let content_end_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(content_end_pos + CRLF_LEN);

        Ok(Self::new(String::from_utf8_lossy(
            &data[1..content_end_pos],
        )))
    }
}

// - bulk error: "!<length>\r\n<error>\r\n"
impl RespDecode for RespBulkError {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;

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

// - integer: ":[<+|->]<value>\r\n"
impl RespDecode for RespInteger {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end_content_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;

        let data = buf.split_to(end_content_pos + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end_content_pos]);
        Ok(RespInteger::new(s.trim().parse()?))
    }
}

// - bulk string: "$<length>\r\n<data>\r\n"
// - null bulk string: "$-1\r\n"
impl RespDecode for RespBulkString {
    fn decode(buf: &mut BytesMut) -> std::result::Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;

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

impl RespDecode for RespNullBulkString {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        if buf == "$-1\r\n" {
            buf.advance(5);
            Ok(Self)
        } else {
            Err(RespDecodeError::InvalidFrame(
                "Invalid RespNullBulkString".to_string(),
            ))
        }
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
//    - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"
// - null array: "*-1\r\n"
impl RespDecode for RespArray {
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

// - null: "_\r\n"
impl RespDecode for RespNull {
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

// - boolean: "#<t|f>\r\n"
impl RespDecode for bool {
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

// - double: ",[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n"
impl RespDecode for f64 {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let end_content_pos = extract_simple_frame_data(buf, Self::FIRST_BYTE)?;
        let data = buf.split_to(end_content_pos + CRLF_LEN);
        let s = String::from_utf8_lossy(&data[1..end_content_pos]);
        Ok(s.trim().parse()?)
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespDecode for RespMap {
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError> {
        let (length_end_pos, length) =
            parse_length(buf, &String::from_utf8_lossy(&Self::FIRST_BYTE))?;
        buf.advance(length_end_pos + CRLF_LEN);

        let mut frames = Self::new();
        for _ in 0..length {
            let key = RespSimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key, value);
        }
        Ok(frames)
    }
}

// - set: "~<number-of-elements>\r\n<element-1>...<element-n>"
impl RespDecode for RespSet {
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

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use bytes::BufMut;

    use crate::RespNullBulkString;

    use super::*;

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

    #[test]
    fn test_simple_error_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"-Error\r\n");

        let frame: RespSimpleError = RespSimpleError::decode(&mut buf).unwrap();
        assert_eq!(frame, RespSimpleError::new("Error".to_string()));
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
        let frame = RespNullBulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, RespNullBulkString);
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

    #[test]
    fn test_map_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespMap::decode(&mut buf).unwrap();
        let mut resp_map = RespMap::new();
        resp_map.insert(
            RespSimpleString::new("hello".to_string()),
            RespBulkString::new(b"world").into(),
        );
        resp_map.insert(
            RespSimpleString::new("foo".to_string()),
            RespBulkString::new(b"bar").into(),
        );
        assert_eq!(frame, resp_map);
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
