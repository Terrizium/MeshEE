#![allow(unused)]

use libp2p::{identity::Keypair, PeerId};

use crate::p2p::error::P2pError;

pub fn get_peer_id_from_device_id(device_id: &str) -> Result<Keypair, P2pError> {
    let seed_bytes = hex::decode(device_id).map_err(|_| P2pError::InvalidDeviceId)?;
    let seed: [u8; 32] = seed_bytes
        .try_into()
        .map_err(|_| P2pError::InvalidDeviceId)?;
    Keypair::ed25519_from_bytes(seed).map_err(|_| P2pError::LibP2pError)
}

#[cfg(test)]
mod tests {
    use serde_json::to_string;

    use super::*;

    #[test]
    fn test_consistent_peer_id_from_device_id() {
        let identity_string = blake3::hash(b"hash_for_device_id").to_hex().to_string();
        let equal_identity_string = blake3::hash(b"hash_for_device_id").to_hex().to_string();
        let another_identity_string = blake3::hash(b"hash_for_device_id2").to_hex().to_string();

        let peer_id = get_peer_id_from_device_id(&identity_string)
            .expect("Error on generate peer id")
            .public()
            .to_peer_id();
        let peer_id2 = get_peer_id_from_device_id(&equal_identity_string)
            .expect("Error on generate peer id")
            .public()
            .to_peer_id();
        let peer_id3 = get_peer_id_from_device_id(&another_identity_string)
            .expect("Error on generate peer id")
            .public()
            .to_peer_id();

        assert_eq!(peer_id, peer_id2);
        assert_ne!(peer_id, peer_id3);
        assert!(!peer_id.to_string().is_empty());
    }
}
