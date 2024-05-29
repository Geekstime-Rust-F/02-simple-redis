use crate::{RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandGet, CommandSet};

impl TryFrom<RespArray> for CommandGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(key)) => {
                Ok(CommandGet::new(String::from_utf8_lossy(&key).to_string()))
            }
            _ => Err(CommandError::InvalidCommandArguments(
                "GET command argument must be a bulk string".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for CommandSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(CommandSet {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidCommandArguments(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};
    use bytes::BytesMut;

    use crate::{
        cmd::{CommandGet, CommandSet},
        RespArray, RespBulkString, RespDecode, RespFrame,
    };

    #[test]
    fn test_get_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let command = CommandGet::try_from(frame).unwrap();
        assert_eq!(command.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let command: CommandSet = frame.try_into()?;
        assert_eq!(command.key, "hello");
        assert_eq!(
            command.value,
            RespFrame::BulkString(RespBulkString::new(b"world".to_vec()))
        );

        Ok(())
    }
}
