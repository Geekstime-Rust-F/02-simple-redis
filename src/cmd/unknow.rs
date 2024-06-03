use crate::{backend::Backend, RespFrame};

use super::{CommandExecutor, RESP_UNKNOWNN_COMMAND};

#[derive(Debug, PartialEq)]
pub struct CommandUnknown;

impl CommandExecutor for CommandUnknown {
    fn execute(self, _backend: &Backend) -> RespFrame {
        RESP_UNKNOWNN_COMMAND.to_owned()
    }
}
