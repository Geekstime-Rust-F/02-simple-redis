use tracing::info;

use crate::{backend::Backend, RespArray, RespBulkString, RespFrame, RespNull};

use super::{extract_args, validate_command, CommandError, CommandExecutor, RESP_OK};

#[derive(Debug, PartialEq)]
pub struct CommandHGet {
    key: String,
    field: String,
}

#[derive(Debug, PartialEq)]
pub struct CommandHSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug, PartialEq)]
pub struct CommandHGetAll {
    key: String,
    sort: bool,
}

#[derive(Debug, PartialEq)]
pub struct CommandHMGet {
    key: String,
    fields: Vec<String>,
}

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
        match backend.hget(&self.key, &self.field) {
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

impl TryFrom<RespArray> for CommandHMGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let n_args = value.len() - 1;
        validate_command(&value, &["hmget"], n_args)?;
        let mut args = extract_args(value, 1)?.into_iter();

        match args.next() {
            Some(RespFrame::BulkString(key)) => {
                let mut string_fields: Vec<String> = Vec::new();
                args.for_each(|field| match field {
                    RespFrame::BulkString(field) => {
                        string_fields.push(String::from_utf8(field.0).unwrap())
                    }
                    _ => {
                        info!("unexpected hmget all field: {:?}", field);
                    }
                });
                if string_fields.len() != n_args - 1 {
                    return Err(CommandError::InvalidCommandArguments(
                        "Invalid hmget field".to_string(),
                    ));
                }

                Ok(CommandHMGet {
                    key: String::from_utf8(key.0)?,
                    fields: string_fields,
                })
            }
            err => Err(CommandError::InvalidCommandArguments(format!(
                "Invalid key or field: {:?}",
                err
            ))),
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
                key: String::from_utf8(field.0)?,
                sort: false,
            }),
            _ => Err(CommandError::InvalidCommandArguments(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl CommandExecutor for CommandHGetAll {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);

        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().to_owned()));
                }

                if self.sort {
                    data.sort_by(|a, b| a.0.cmp(&b.0));
                }
                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![RespBulkString::from(k).into(), v])
                    .collect::<Vec<RespFrame>>();
                RespArray::new(ret).into()
            }
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for CommandHMGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);

        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(self.fields.len());
                for v in self.fields {
                    if let Some(v) = hmap.get(&v) {
                        data.push(v.value().to_owned());
                    } else {
                        data.push(RespFrame::Null(RespNull));
                    }
                }
                RespArray::new(data).into()
            }
            None => RespFrame::Null(RespNull),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{
            hmap::{CommandHGet, CommandHGetAll, CommandHMGet, CommandHSet},
            CommandExecutor,
        },
        RespArray, RespBulkString, RespFrame, RespNull,
    };
    use anyhow::{Ok, Result};

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
        assert_eq!(hgetall_command.key, "map");

        Ok(())
    }
    #[test]
    fn test_hmget_command_from_resp_array() -> Result<()> {
        let backend = crate::backend::Backend::new();
        backend.hset("map", "hello", RespBulkString::new("world").into());
        backend.hset("map", "hello2", RespBulkString::new("world2").into());

        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hmget".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello2".to_vec())),
        ]);

        let hmget_command: CommandHMGet = resp_array.try_into()?;

        assert_eq!(hmget_command.key, "map");
        assert_eq!(hmget_command.fields, vec!["hello", "hello2"]);

        Ok(())
    }

    #[test]
    fn test_hgetall_execute() -> Result<()> {
        let backend = crate::backend::Backend::new();
        backend.hset("map", "hello", RespBulkString::new("world").into());

        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hgetall".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
        ]);
        let hgetall_command: CommandHGetAll = resp_array.try_into()?;
        let resp_frame = hgetall_command.execute(&backend);
        assert_eq!(
            resp_frame,
            RespArray::new(vec![
                RespFrame::BulkString(RespBulkString::new(b"hello".to_vec())),
                RespFrame::BulkString(RespBulkString::new(b"world".to_vec())),
            ])
            .into()
        );

        Ok(())
    }

    #[test]
    fn test_hmget_execute() -> Result<()> {
        let backend = crate::backend::Backend::new();
        backend.hset("map", "hello", RespBulkString::new("world").into());
        backend.hset("map", "hello2", RespBulkString::new("world2").into());

        let resp_array = RespArray::new(vec![
            RespFrame::BulkString(RespBulkString::new(b"hmget".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"map".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello2".to_vec())),
            RespFrame::BulkString(RespBulkString::new(b"hello3".to_vec())),
        ]);

        let hmget_command: CommandHMGet = resp_array.try_into()?;
        let resp_frame = hmget_command.execute(&backend);
        assert_eq!(
            resp_frame,
            RespArray::new(vec![
                RespFrame::BulkString(RespBulkString::new(b"world".to_vec())),
                RespFrame::BulkString(RespBulkString::new(b"world2".to_vec())),
                RespFrame::Null(RespNull),
            ])
            .into()
        );

        Ok(())
    }
}
