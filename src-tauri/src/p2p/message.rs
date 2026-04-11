#![allow(unused)]

use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::{
    multihash::derive,
    request_response::{self, ProtocolName},
};
use serde::{Deserialize, Serialize};
use std::io; // 👈 futures::io

use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub sender: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct ChatProtocol;

impl ProtocolName for ChatProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/meshee-chat/1.0"
    }
}

#[derive(Clone)]
pub struct JsonCodec;

#[async_trait]
impl request_response::Codec for JsonCodec {
    type Protocol = ChatProtocol;
    type Request = ChatMessage;
    type Response = ChatMessage;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send, // 👈 futures::io::AsyncRead + Send
    {
        let mut buf = Vec::new();
        AsyncReadExt::read_to_end(io, &mut buf)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        AsyncReadExt::read_to_end(io, &mut buf)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf =
            serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        AsyncWriteExt::write_all(io, &buf)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf =
            serde_json::to_vec(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        AsyncWriteExt::write_all(io, &buf)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
