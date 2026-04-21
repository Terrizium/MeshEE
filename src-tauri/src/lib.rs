#![allow(unused)]

mod auth;
mod p2p;

use libp2p::PeerId;
use serde_json::json;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::auth::Profile;
use crate::auth::ProfileData;
use crate::p2p::ChatMessage;

pub struct P2pHandle {
    pub tx: mpsc::UnboundedSender<ChatMessage>,
    pub task: JoinHandle<()>,
    pub local_peer_id: PeerId,
}

pub struct AppState {
    pub cur_profile: Mutex<Option<Profile>>,
    pub p2p_handle: Mutex<Option<P2pHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cur_profile: Mutex::new(None),
            p2p_handle: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
async fn init_p2p<R: tauri::Runtime>(
    device_id: String,
    app_handle: tauri::AppHandle<R>,
) -> Result<String, String> {
    let keypair = p2p::identity::get_peer_id_from_device_id(&device_id)
        .map_err(|e| format!("Invalid device_id: {}", e))?;
    let peer_id = PeerId::from(keypair.public());
    let (mut swarm, tx_loop, _rx) = p2p::swarm::create_swarm_with_rx(keypair)
        .await
        .map_err(|e| format!("Failed to create swarm: {}", e))?;
    swarm
        .listen_on(
            "/ip4/0.0.0.0/tcp/0"
                .parse::<libp2p::Multiaddr>()
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("Failed to listen: {}", e))?;
    let tx_send = tx_loop.clone();
    let task = tokio::spawn(async move {
        p2p::swarm::run_swarm_loop(swarm, tx_loop).await;
    });
    let state = app_handle.state::<AppState>();
    let mut lock = state.p2p_handle.lock().unwrap();
    *lock = Some(P2pHandle {
        tx: tx_send,
        task,
        local_peer_id: peer_id,
    });
    Ok(peer_id.to_string())
}

#[tauri::command]
async fn start_periodic_messages(app_handle: tauri::AppHandle) {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3));
        let mut counter = 0;
        loop {
            interval.tick().await;
            counter += 1;
            let message = format!("Автоматическое сообщение #{}", counter);
            let _ = app_handle.emit(
                "new-message",
                &json!({
                    "id": counter,
                    "msg": message
                }),
            );
        }
    });
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn login(name: &str, password: &str, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    match auth::login(name, password) {
        Ok(profile) => {
            //TODO:: Если переходить на tokio, то +async и заменить Mutex
            let mut lock = state.cur_profile.lock().unwrap();
            *lock = Some(profile);
            Ok(true)
        }
        Err(err) => Err(format!("Ошибка входа: {}", err.to_string())),
    }
}

#[tauri::command]
async fn load(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ProfileData, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let username = state
        .cur_profile
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .username
        .clone();
    match auth::load(&username, &data_dir).await {
        Ok(data) => Ok(data),
        Err(err) => Err(format!("Failed to load data: {}", err)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            start_periodic_messages,
            init_p2p
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_app, mock_builder, mock_context, noop_assets};

    #[tokio::test]
    async fn init_p2p_returns_valid_peer_id_and_updates_state() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let device_id = "0000000000000000000000000000000000000000000000000000000000000001";
        let result =
            init_p2p::<tauri::test::MockRuntime>(device_id.to_string(), app.app_handle().clone())
                .await;
        assert!(result.is_ok());
        let peer_id_str = result.unwrap();
        assert!(!peer_id_str.is_empty());
        let state = app.state::<AppState>();
        let handle_lock = state.p2p_handle.lock().unwrap();
        assert!(handle_lock.is_some());
        let handle = handle_lock.as_ref().unwrap();
        assert_eq!(handle.local_peer_id.to_string(), peer_id_str);
    }
}
