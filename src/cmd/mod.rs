mod echo;
mod hmap;
mod map;
mod unknow;

use echo::CommandEcho;
use enum_dispatch::enum_dispatch;
use hmap::{CommandHGet, CommandHGetAll, CommandHSet};
use lazy_static::lazy_static;
use map::{CommandGet, CommandSet};
use std::string::FromUtf8Error;
use thiserror::Error;
use unknow::CommandUnknown;

use crate::{
    backend::Backend, RespArray, RespDecodeError, RespFrame, RespSimpleError, RespSimpleString,
};

lazy_static! {
    static ref RESP_OK: RespFrame =
        RespFrame::SimpleString(RespSimpleString::new("OK".to_string()));
    static ref RESP_UNKNOWNN_COMMAND: RespFrame =
        RespFrame::Error(RespSimpleError::new("Unknown command".to_string()));
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid command arguments: {0}")]
    InvalidCommandArguments(String),

    #[error("{0}")]
    RespError(#[from] RespDecodeError),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

#[derive(Debug, PartialEq)]
#[enum_dispatch(CommandExecutor)]
pub enum Command {
    Get(CommandGet),
    Set(CommandSet),
    HGet(CommandHGet),
    HSet(CommandHSet),
    HGetAll(CommandHGetAll),

    Echo(CommandEcho),

    // unknown commands
    UnknownCommand(CommandUnknown),
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        match value.first() {
            Some(RespFrame::BulkString(ref command)) => match command.as_ref() {
                b"get" => Ok(CommandGet::try_from(value)?.into()),
                b"set" => Ok(CommandSet::try_from(value)?.into()),
                b"hget" => Ok(CommandHGet::try_from(value)?.into()),
                b"hset" => Ok(CommandHSet::try_from(value)?.into()),
                b"hgetall" => Ok(CommandHGetAll::try_from(value)?.into()),
                b"echo" => Ok(CommandEcho::try_from(value)?.into()),
                _ => Ok(CommandUnknown.into()),
            },
            _ => todo!(),
        }
    }
}

pub fn validate_command(
    value: &RespArray,
    command_names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != command_names.len() + n_args {
        return Err(CommandError::InvalidCommandArguments(format!(
            "GET command must have exactly {} argument",
            n_args
        )));
    }
    for (i, command_name) in command_names.iter().enumerate() {
        match &value[i] {
            RespFrame::BulkString(ref command) => {
                if &*command.to_ascii_lowercase() != command_name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: {:?}",
                        value[i]
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(format!(
                    "Invalid command: {:?}",
                    value[i]
                )));
            }
        }
    }

    Ok(())
}

pub fn extract_args(
    value: RespArray,
    command_length: usize,
) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(command_length).collect())
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{map::CommandGet, validate_command},
        RespArray, RespBulkString, RespFrame,
    };
    use anyhow::Result;

    use super::extract_args;

    #[test]
    fn test_validate_command() {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"get".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"key".to_vec())),
        ]);
        let result = validate_command(&resp_array, &["get"], 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_args() -> Result<()> {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"get".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"key".to_vec())),
        ]);

        let args = extract_args(resp_array, 1)?;
        assert_eq!(args.len(), 1);

        Ok(())
    }

    #[test]
    fn test_command_try_from() -> Result<()> {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"get".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"key".to_vec())),
        ]);

        let command: super::Command = resp_array.try_into()?;
        assert_eq!(
            command,
            super::Command::Get(CommandGet::new("key".to_string()))
        );

        Ok(())
    }
}
