use anyhow::Result;
use bytes::BytesMut;
use tracing::info;

use crate::RespDecodeError;

use super::{
    array::RespArray, bulk_error::RespBulkError, bulk_string::RespBulkString, frame::RespFrame,
    integer::RespInteger, map::RespMap, null::RespNull, set::RespSet,
    simple_error::RespSimpleError, simple_string::RespSimpleString,
};

pub const CRLF_LEN: usize = 2;
pub const CRLF: &str = "\r\n";

pub trait RespFrameFirstByte {
    const FIRST_BYTE: [u8; 1];
}

pub trait RespDecode: Sized {
    const FIRST_BYTE: [u8; 1];
    fn decode(buf: &mut BytesMut) -> Result<Self, RespDecodeError>;
}

impl RespDecode for RespFrame {
    const FIRST_BYTE: [u8; 1] = [b'?'];
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
            Some(b'$') => Ok(RespBulkString::decode(buf)?.into()),
            Some(b'*') => Ok(RespArray::decode(buf)?.into()),
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

pub fn find_nth_crlf(buf: &[u8], nth: usize) -> Option<usize> {
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

pub fn extract_simple_frame_data(
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

pub fn parse_length(buf: &mut BytesMut, prefix: &str) -> Result<(usize, isize), RespDecodeError> {
    let length_end_pos = extract_simple_frame_data(buf, [prefix.as_bytes()[0]])?;
    let length = String::from_utf8_lossy(&buf[prefix.len()..length_end_pos]);
    Ok((length_end_pos, length.parse()?))
}
