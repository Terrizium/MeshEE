#![allow(unused)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("invalid login+password pair")]
    InvalidDeviceId,

    #[error("libp2p error")]
    LibP2pError,
}
