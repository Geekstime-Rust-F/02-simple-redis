use anyhow::{Ok, Result};
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tracing::info;

use crate::resp::RespEncode;
use crate::{
    backend::Backend,
    cmd::{Command, CommandExecutor},
    RespArray, RespDecode, RespFrame,
};
use tokio_util::codec::{Decoder, Encoder, Framed};

struct RespFrameCodec;

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<()> {
        let encoded = item.encode()?;
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}

impl Decoder for RespFrameCodec {
    type Item = RespArray;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>> {
        info!(
            "RespFrameCodec decode command buf: {}",
            String::from_utf8_lossy(src)
        );
        if src.is_empty() {
            return Ok(None);
        }
        let frame = RespArray::decode(src)?;
        Ok(Some(frame))
    }
}

#[derive(Debug)]
struct RedisRequest {
    frame: RespArray,
    backend: Backend,
}

#[derive(Debug)]
struct RedisResponse {
    response: RespFrame,
}

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(stream, RespFrameCodec);

    loop {
        match framed.next().await {
            Some(std::result::Result::Ok(frame)) => {
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                info!("request: {:?}", request);
                let response = request_handler(request).await?;
                framed.send(response.response).await?;
            }
            Some(Err(err)) => return Err(err),
            None => return Ok(()),
        }
    }
}

async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let cmd = Command::try_from(frame)?;
    let ret = cmd.execute(&backend);
    Ok(RedisResponse { response: ret })
}
