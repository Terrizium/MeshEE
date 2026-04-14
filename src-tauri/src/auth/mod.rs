#![allow(unused)]

use std::path::Path;

pub use crate::auth::profile::Profile;
pub use crate::auth::storage::ProfileData;

pub mod error;
mod profile;
mod storage;

pub fn login(username: &str, password: &str) -> Result<Profile, error::AuthError> {
    profile::Profile::new(username.to_string(), password.to_string())
}

pub async fn load(username: &str, data_dir: &Path) -> Result<ProfileData, error::AuthError> {
    storage::load(username.to_string(), data_dir).await
}

pub async fn save(profile_data: ProfileData, data_dir: &Path) -> Result<(), error::AuthError> {
    storage::save(profile_data, data_dir).await
}

#[cfg(test)]
mod tests {
    use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
    use tempfile::tempdir;

    use crate::{
        auth::{load, save, Profile, ProfileData},
        p2p::identity::get_peer_id_from_device_id,
    };

    #[tokio::test]
    async fn test_address_book_persists_across_save_load() {
        let tmp_dir = tempdir().unwrap();
        let username = "user_1".to_string();
        let password = "secret123".to_string();

        let profile = Profile::new(username.clone(), password.clone()).unwrap();
        let device_id = &profile.device_id;
        let kp = get_peer_id_from_device_id(device_id).unwrap();
        let my_peer_id = PeerId::from(kp.public());

        let remote_peer_id = PeerId::random();
        let remote_addr: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
        let remote_addr_with_peer = remote_addr
            .clone()
            .with(Protocol::P2p(remote_peer_id.into()));

        let mut profile_data = ProfileData::default();
        profile_data.username = username.clone();
        profile_data.peer_id = my_peer_id.to_string();
        profile_data.add_known_peer(remote_peer_id, remote_addr_with_peer.clone());

        save(profile_data, tmp_dir.path()).await.unwrap();

        let loaded = load(&username.clone(), tmp_dir.path()).await.unwrap();
        let known = loaded.get_known_peers(remote_peer_id);

        assert_eq!(known.len(), 1);
        assert!(known.contains(&remote_addr_with_peer));
    }

    #[tokio::test]
    async fn test_profile_load_restores_peer_id_correctly() {
        let tmp_dir = tempdir().unwrap();
        let username = "user_1".to_string();
        let password = "secret123".to_string();

        let profile = Profile::new(username.clone(), password.clone()).unwrap();
        let device_id = &profile.device_id;
        let kp_original = get_peer_id_from_device_id(device_id).unwrap();
        let original_peer_id = PeerId::from(kp_original.public());

        let mut profile_data = ProfileData::default();
        profile_data.username = username.clone();
        profile_data.peer_id = original_peer_id.to_string();
        save(profile_data.clone(), tmp_dir.path()).await.unwrap();

        let loaded_profile = Profile::new(username.clone(), password.clone()).unwrap();
        let loaded_device_id = &loaded_profile.device_id;
        let kp_loaded = get_peer_id_from_device_id(loaded_device_id).unwrap();
        let loaded_peer_id = PeerId::from(kp_loaded.public());

        assert_eq!(original_peer_id, loaded_peer_id);
        assert_eq!(profile_data.peer_id, loaded_peer_id.to_string());

        let wrong_password = "wrong".to_string();
        let wrong_profile = Profile::new(username.clone(), wrong_password).unwrap();
        let wrong_device_id = &wrong_profile.device_id;
        let kp_wrong = get_peer_id_from_device_id(wrong_device_id).unwrap();
        let wrong_peer_id = PeerId::from(kp_wrong.public());
        assert_ne!(original_peer_id, wrong_peer_id);
    }
}
