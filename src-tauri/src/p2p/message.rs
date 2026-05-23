#![allow(unused)]

use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::{
    identity::{Keypair, PublicKey},
    multihash::derive,
    request_response::{self, ProtocolName},
};
use serde::{Deserialize, Serialize};
use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
}; // 👈 futures::io

use uuid::Uuid;

use crate::p2p::error::ChatError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    #[serde(default)]
    pub sender_public_key: Vec<u8>,
    pub timestamp: u64,
    #[serde(default)]
    pub signature: Vec<u8>,
}

impl ChatMessage {
    pub fn new(sender: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            sender_public_key: Vec::new(),
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: Vec::new(),
        }
    }
    pub fn sign(mut self, kp: &Keypair) -> Self {
        self.sender_public_key = kp.public().encode_protobuf();
        let mut msg_to_sign = self.clone();
        msg_to_sign.signature = Vec::new();

        let data = serde_json::to_vec(&msg_to_sign).expect("Message should serialize");
        self.signature = kp.sign(&data).expect("Keypair should sign data");
        self
    }

    pub fn verify_signature(&self) -> Result<(), ChatError> {
        if self.signature.is_empty() {
            return Err(ChatError::NoSignatureError);
        }
        let mut msg_to_verify = self.clone();
        msg_to_verify.signature = Vec::new();

        let data = serde_json::to_vec(&msg_to_verify).map_err(|e| ChatError::SerializeError)?;
        let pubkey = PublicKey::try_decode_protobuf(&self.sender_public_key)
            .map_err(|e| ChatError::PublicKeyError)?;
        let res = pubkey.verify(&data, &self.signature);
        if !res {
            return Err(ChatError::VerificationFailedError);
        }
        Ok(())
    }
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
