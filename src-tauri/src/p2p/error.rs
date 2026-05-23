#![allow(unused)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("invalid login+password pair")]
    InvalidDeviceId,

    #[error("libp2p error")]
    LibP2pError,
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Validation error: No signature")]
    NoSignatureError,
    #[error("Validation error: Can't serialize")]
    SerializeError,
    #[error("Validation error: Wrong PublicKey")]
    PublicKeyError,
    #[error("Validation error: Signature verification failed")]
    VerificationFailedError,
}
