use crate::{backend::Backend, RespArray, RespBulkString, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor};

#[derive(Debug, PartialEq)]
pub struct CommandEcho {
    value: String,
}
impl CommandEcho {
    fn new(value: String) -> Self {
        Self { value }
    }
}

impl CommandExecutor for CommandEcho {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RespBulkString::from(self.value).into()
    }
}

impl TryFrom<RespArray> for CommandEcho {
    type Error = CommandError;

    fn try_from(frame: RespArray) -> Result<Self, Self::Error> {
        validate_command(&frame, &["echo"], 1)?;
        let mut args = extract_args(frame, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(value)) => Ok(CommandEcho::new(
                String::from_utf8_lossy(&value).to_string(),
            )),
            _ => Err(CommandError::InvalidCommandArguments(
                "Echo command argument must be a bulk string".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Ok, Result};
    use bytes::BytesMut;

    use crate::{cmd::echo::CommandEcho, RespArray, RespDecode};

    #[test]
    fn test_echo_command_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        let command = CommandEcho::try_from(frame).unwrap();
        assert_eq!(command.value, "hello");

        Ok(())
    }
}
