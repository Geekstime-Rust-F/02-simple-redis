mod hmap;
mod map;

use lazy_static::lazy_static;
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::{backend::Backend, RespArray, RespDecodeError, RespFrame, RespSimpleString};

lazy_static! {
    static ref RESP_OK: RespFrame =
        RespFrame::SimpleString(RespSimpleString::new("OK".to_string()));
}

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

pub enum Command {
    Get(CommandGet),
    Set(CommandSet),
    HGet(CommandHGet),
    HSet(CommandHGetAll),
}

#[derive(Debug)]
pub struct CommandGet {
    key: String,
}
impl CommandGet {
    fn new(key: String) -> Self {
        Self { key }
    }
}

#[derive(Debug)]
pub struct CommandSet {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct CommandHGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct CommandHSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct CommandHGetAll {
    field: String,
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(_value: RespArray) -> Result<Self, Self::Error> {
        todo!()
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
    use crate::{cmd::validate_command, RespArray, RespBulkString, RespFrame};
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
}
