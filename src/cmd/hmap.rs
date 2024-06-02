use crate::{RespArray, RespFrame, RespNull};

use super::{
    extract_args, validate_command, CommandError, CommandExecutor, CommandHGet, CommandHGetAll,
    CommandHSet, RESP_OK,
};

impl TryFrom<RespArray> for CommandHGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => {
                Ok(CommandHGet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                })
            }
            _ => Err(CommandError::InvalidCommandArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl CommandExecutor for CommandHGet {
    fn execute(self, backend: &crate::backend::Backend) -> RespFrame {
        match backend.hsget(&self.key, &self.field) {
            Some(value) => value,
            None => RespFrame::Null(RespNull),
        }
    }
}

impl TryFrom<RespArray> for CommandHSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match (args.next(), args.next(), args.next()) {
            (
                Some(RespFrame::BulkString(key)),
                Some(RespFrame::BulkString(field)),
                Some(RespFrame::BulkString(value)),
            ) => Ok(CommandHSet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
                value: RespFrame::BulkString(value),
            }),
            _ => Err(CommandError::InvalidCommandArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}
impl CommandExecutor for CommandHSet {
    fn execute(self, backend: &crate::backend::Backend) -> RespFrame {
        backend.hset(&self.key, &self.field, self.value);
        RESP_OK.to_owned()
    }
}

impl TryFrom<RespArray> for CommandHGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(field)) => Ok(CommandHGetAll {
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidCommandArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl CommandExecutor for CommandHGetAll {
    fn execute(self, _backend: &crate::backend::Backend) -> RespFrame {
        // backend.hgetall(&self.field).into()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{CommandHGet, CommandHGetAll, CommandHSet},
        RespArray, RespBulkString, RespFrame,
    };
    use anyhow::Result;

    #[test]
    fn test_hget_command_from_resp_array() -> Result<()> {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hget".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello".to_vec())),
        ]);
        let hget_command: CommandHGet = resp_array.try_into()?;
        assert_eq!(hget_command.key, "map");
        assert_eq!(hget_command.field, "hello");

        Ok(())
    }

    #[test]
    fn test_hset_command_from_resp_array() -> Result<()> {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hset".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"world".to_vec())),
        ]);
        let hset_command: CommandHSet = resp_array.try_into()?;
        assert_eq!(hset_command.key, "map");
        assert_eq!(hset_command.field, "hello");
        assert_eq!(
            hset_command.value,
            RespFrame::BulkString(RespBulkString::new(b"world".to_vec()))
        );

        Ok(())
    }

    #[test]
    fn test_hgetall_command_from_resp_array() -> Result<()> {
        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hgetall".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
        ]);
        let hgetall_command: CommandHGetAll = resp_array.try_into()?;
        assert_eq!(hgetall_command.field, "map");

        Ok(())
    }
}
