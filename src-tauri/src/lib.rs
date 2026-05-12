#![allow(unused)]

mod auth;
mod p2p;

// use futures::channel::mpsc;
use libp2p::identity::Keypair;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::format;
use std::path::Path;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::auth::Profile;
use crate::auth::ProfileData;
use crate::p2p::ChatMessage;
use crate::p2p::P2pCommand;

//TODO:: drop device_id from ProfileData

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub login: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatInfo {
    pub id: String,
    pub login: String,
    pub has_unread: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageInfo {
    pub id: String,
    pub body: String,
    pub is_read: bool,
    pub date: String,
    pub user_id: u32, // 0 - user, 1 - remote_user
}

#[derive(Serialize, Deserialize)]
pub struct PaginatedMessages {
    pub meta: PageMeta,
    pub messages: Vec<MessageInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct PageMeta {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
}

pub async fn handle_incoming_message<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    msg: ChatMessage,
) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let username = state
        .cur_profile
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?
        .as_ref()
        .ok_or("No active profile")?
        .username
        .clone();
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    process_and_save_incoming_message(&msg, &username, &data_dir).await?;
    let _ = app_handle.emit(
        "new-message",
        &serde_json::json!({
            "chat_id": msg.sender,
            "message": {
                "id": msg.id,
                "body": msg.content,
                "is_read": false,
                "date": chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
                    .unwrap_or_default()
                    .to_rfc3339(),
                "user_id": 1,
            }
        }),
    );
    Ok(())
}

pub async fn process_and_save_incoming_message(
    msg: &ChatMessage,
    username: &str,
    data_dir: &Path,
) -> Result<(), String> {
    msg.verify_signature()
        .map_err(|e| format!("Invalid signature: {}", e))?;
    let mut proifle = auth::load(username, data_dir)
        .await
        .map_err(|e| format!("Failed to load proifle: {}", e))?;
    let auth_msg = auth::Message {
        username: msg.sender.clone(),
        text: msg.content.clone(),
        created_at: chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
            .unwrap_or_default()
            .to_rfc3339(),
        received_at: chrono::Local::now().to_rfc3339(),
    };
    if let Some(chat) = proifle.chats.iter_mut().find(|c| c.name == msg.sender) {
        chat.messages.push(auth_msg);
    } else {
        proifle.chats.push(auth::Chat {
            name: msg.sender.clone(),
            peer_id: msg.sender.clone(),
            messages: vec![auth_msg],
        });
    }
    auth::save(proifle, data_dir)
        .await
        .map_err(|e| format!("Failed to save profile: {}", e))?;
    Ok(())
}

pub struct P2pHandle {
    pub cmd_tx: mpsc::UnboundedSender<P2pCommand>,
    // pub event_rx: Mutex<mpsc::UnboundedReceiver<ChatMessage>>,
    pub task: JoinHandle<()>,
    pub local_peer_id: PeerId,
    pub keypair: Keypair,
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
async fn connect_to_peer<R: tauri::Runtime>(
    peer_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<ChatInfo, String> {
    let username = {
        let lock = state
            .cur_profile
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let profile = lock.as_ref().ok_or("No user logged in")?;
        profile.username.clone()
    };
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let mut profile_data = auth::load(&username, &data_dir)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(existing_chat) = profile_data.chats.iter_mut().find(|c| c.peer_id == peer_id) {
        return Ok(ChatInfo {
            id: existing_chat.name.clone(),
            login: existing_chat.name.clone(),
            has_unread: false,
        });
    }
    profile_data.chats.push(auth::Chat {
        name: peer_id.clone(),
        peer_id: peer_id.clone(),
        messages: Vec::new(),
    });
    auth::save(profile_data, &data_dir)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ChatInfo {
        id: peer_id.clone(),
        login: peer_id.clone(),
        has_unread: false,
    })
}

#[tauri::command]
async fn send_message<R: tauri::Runtime>(
    chat_id: String,
    text: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<MessageInfo, String> {
    let username = {
        let lock = state
            .cur_profile
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let profile = lock.as_ref().ok_or("No user logged in")?;
        profile.username.clone()
    };
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let mut profile_data = auth::load(&username, &data_dir)
        .await
        .map_err(|e| e.to_string())?;
    let chat = profile_data
        .chats
        .iter_mut()
        .find(|c| c.name == chat_id)
        .ok_or("Chat not found")?;
    let peer_id = chat
        .peer_id
        .parse::<PeerId>()
        .map_err(|e| format!("Invalid peeer id in chat: {}", e))?;

    let cmd_tx = {
        let handle_lock = state
            .p2p_handle
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let handle = handle_lock.as_ref().ok_or("P2p not initialized")?;
        handle.cmd_tx.clone()
    };
    let (local_peer_id, keypair) = {
        let handle_lock = state
            .p2p_handle
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let handle = handle_lock.as_ref().ok_or("P2p not initialized")?;
        (handle.local_peer_id.clone(), handle.keypair.clone())
    };
    let msg = ChatMessage::new(local_peer_id.to_string(), text.clone()).sign(&keypair);
    cmd_tx
        .send(P2pCommand::SendMessage {
            peer_id,
            message: msg.clone(),
        })
        .map_err(|e| format!("Failed to send P2p message: {}", e))?;
    let auth_msg = auth::Message {
        username: username.clone(),
        text: msg.content.clone(),
        created_at: chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
            .unwrap_or_default()
            .to_rfc3339(),
        received_at: chrono::Local::now().to_rfc3339(),
    };
    chat.messages.push(auth_msg);
    auth::save(profile_data, &data_dir)
        .await
        .map_err(|e| e.to_string())?;
    Ok(MessageInfo {
        id: msg.id,
        body: msg.content,
        is_read: true,
        date: chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
            .unwrap_or_default()
            .to_rfc3339(),
        user_id: 0,
    })
}

#[tauri::command]
async fn get_chats<R: tauri::Runtime>(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<Vec<ChatInfo>, String> {
    let username = {
        let lock = state
            .cur_profile
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let profile = lock.as_ref().ok_or("No user logged in")?;
        profile.username.clone()
    };
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let profile_data = auth::load(&username, &data_dir)
        .await
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for chat in profile_data.chats {
        let has_unread = chat
            .messages
            .iter()
            .any(|m| m.received_at.is_empty() && m.username != username);
        result.push(ChatInfo {
            id: chat.name.clone(),
            login: chat.name.clone(),
            has_unread,
        });
    }
    Ok(result)
}

#[tauri::command]
async fn get_chat<R: tauri::Runtime>(
    chat_id: String,
    page: usize,
    per_page: usize,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<PaginatedMessages, String> {
    let username = {
        let lock = state
            .cur_profile
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let profile = lock.as_ref().ok_or("No user logged in")?;
        profile.username.clone()
    };
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let profile_data = auth::load(&username, &data_dir)
        .await
        .map_err(|e| e.to_string())?;
    let chat = profile_data
        .chats
        .iter()
        .find(|c| c.name == chat_id)
        .ok_or("Chat not found")?;
    let mut messages: Vec<MessageInfo> = chat
        .messages
        .iter()
        .map(|m| MessageInfo {
            id: format!("{}-{}", m.created_at, m.username),
            body: m.text.clone(),
            is_read: !m.received_at.is_empty(),
            date: m.created_at.clone(),
            user_id: if m.username == username { 0 } else { 1 },
        })
        .collect();
    messages.sort_by(|a, b| b.date.cmp(&a.date));
    let total = messages.len();
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(total);
    let paginated = if start < total {
        messages[start..end].to_vec()
    } else {
        vec![]
    };
    Ok(PaginatedMessages {
        meta: PageMeta {
            page,
            per_page,
            total,
        },
        messages: paginated,
    })
}

#[tauri::command]
fn get_local_peer_id(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let handle_lock = state
        .p2p_handle
        .lock()
        .map_err(|e| format!("Failed to lock state: {}", e))?;
    let handle = handle_lock
        .as_ref()
        .ok_or("P2P not initialized: call init_p2p first")?;
    Ok(handle.local_peer_id.to_string())
}

#[tauri::command]
async fn send_p2p_message<R: tauri::Runtime>(
    peer_id_str: String,
    content: String,
    app_handle: tauri::AppHandle<R>,
) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let handle_lock = state.p2p_handle.lock().unwrap();
    let handle = handle_lock
        .as_ref()
        .ok_or("P2P not initialized: call init_p2p first")?;
    let peer_id = peer_id_str
        .parse::<PeerId>()
        .map_err(|e| format!("Invalid peer_id format: {}", e))?;
    let msg = ChatMessage::new(handle.local_peer_id.to_string(), content).sign(&handle.keypair);
    handle
        .cmd_tx
        .send(P2pCommand::SendMessage {
            peer_id,
            message: msg,
        })
        .map_err(|e| format!("Failed to queue message: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn init_p2p<R: tauri::Runtime>(
    device_id: String,
    app_handle: tauri::AppHandle<R>,
) -> Result<String, String> {
    let keypair = p2p::identity::get_peer_id_from_device_id(&device_id)
        .map_err(|e| format!("Invalid device_id: {}", e))?;
    let peer_id = PeerId::from(keypair.public());
    let (mut swarm, cmd_tx, cmd_rx, event_tx, mut event_rx) =
        p2p::swarm::create_swarm_with_rx(keypair.clone())
            .await
            .map_err(|e| format!("Failed to create swarm: {}", e))?;
    swarm
        .listen_on(
            "/ip4/0.0.0.0/tcp/0"
                .parse::<libp2p::Multiaddr>()
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("Failed to listen: {}", e))?;
    let state = app_handle.state::<AppState>();
    let handle_clone = app_handle.clone();
    let task = tokio::spawn(async move {
        let swarm_task = tokio::spawn(async move {
            p2p::swarm::run_swarm_loop(swarm, cmd_rx, event_tx).await;
        });
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = event_rx.recv().await {
                if let Err(e) = handle_incoming_message(handle_clone.clone(), msg).await {
                    eprintln!("Failed to handle incoming message: {}", e);
                }
            }
        });
        let _ = tokio::join!(swarm_task, incoming_task);
    });
    let mut lock = state.p2p_handle.lock().unwrap();
    *lock = Some(P2pHandle {
        cmd_tx,
        task,
        local_peer_id: peer_id,
        keypair,
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
async fn get_user<R: tauri::Runtime>(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<User, String> {
    let lock = state
        .cur_profile
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    let profile = lock.as_ref().ok_or("No user logged in")?;
    Ok(User {
        id: profile.username.clone(),
        login: profile.username.clone(),
    })
}

#[tauri::command]
async fn login<R: tauri::Runtime>(
    name: &str,
    password: &str,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle<R>,
) -> Result<User, String> {
    let username = name.trim();
    if username.is_empty() {
        return Err("Username cannot be empty".into());
    }
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    match auth::load(username, &data_dir).await {
        Ok(mut profile_data) => {
            let test_profile = Profile::new(username.to_string(), password.to_string())
                .map_err(|e| format!("Failed to create profile: {}", e))?;
            let profile = Profile {
                username: username.to_string(),
                device_id: test_profile.device_id.clone(),
            };
            let mut lock = state.cur_profile.lock().unwrap();
            *lock = Some(profile);
            Ok(User {
                id: username.to_string(),
                login: username.to_string(),
            })
        }
        Err(auth::error::AuthError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            let profile = Profile::new(username.to_string(), password.to_string())
                .map_err(|e| format!("Failed to create profile: {}", e))?;
            let mut profile_data = ProfileData::default();
            profile_data.username = username.to_string();
            profile_data.device_id = profile.device_id.clone();
            auth::save(profile_data, &data_dir)
                .await
                .map_err(|e| format!("Failed to save profile: {}", e))?;
            let mut lock = state.cur_profile.lock().unwrap();
            *lock = Some(profile);
            Ok(User {
                id: username.to_string(),
                login: username.to_string(),
            })
        }
        Err(e) => Err(format!("Failed to load profile: {}", e)),
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
            get_user,
            get_chats,
            get_chat,
            connect_to_peer,
            send_message,
            start_periodic_messages,
            init_p2p,
            send_p2p_message,
            get_local_peer_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::p2p::identity::get_peer_id_from_device_id;

    use super::*;
    use tauri::test::{mock_app, mock_builder, mock_context, noop_assets};

    #[tokio::test]
    async fn test_connect_to_peer_creates_new_chat() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "connector";
        let password = "pass";
        login::<tauri::test::MockRuntime>(username, password, app.state(), app_handle.clone())
            .await
            .unwrap();
        let peer_id = "12D3KooW...test".to_string();
        let chat = connect_to_peer::<tauri::test::MockRuntime>(
            peer_id.clone(),
            app.state(),
            app_handle.clone(),
        )
        .await
        .unwrap();
        assert_eq!(chat.login, peer_id);
        assert_eq!(chat.has_unread, false);

        let data_dir = app.app_handle().path().app_data_dir().unwrap();
        let mut profile_data = auth::load(username, &data_dir).await.unwrap();
        let saved_chat = profile_data
            .chats
            .iter()
            .find(|c| c.peer_id == peer_id)
            .unwrap();
        assert_eq!(saved_chat.name, peer_id);
    }

    #[tokio::test]
    async fn test_send_message_sends_via_p2p_and_saves_locally() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "sender";
        let password = "pass";
        login::<tauri::test::MockRuntime>(username, password, app.state(), app_handle.clone())
            .await
            .unwrap();
        // let device_id = {
        //     let profile = auth::load(username, &app.app_handle().path().app_data_dir().unwrap())
        //         .await
        //         .unwrap();
        //     profile.device_id.clone()
        // };
        let device_id = {
            let state = app.state::<AppState>();
            let lock = state.cur_profile.lock().unwrap();
            let profile = lock.as_ref().unwrap();
            profile.device_id.clone()
        };
        init_p2p::<tauri::test::MockRuntime>(device_id, app_handle.clone())
            .await
            .unwrap();
        let local_peer_id = {
            let state = app.state::<AppState>();
            let lock = state.p2p_handle.lock().unwrap();
            let handle = lock.as_ref().unwrap();
            handle.local_peer_id.to_string()
        };
        connect_to_peer::<tauri::test::MockRuntime>(
            local_peer_id.clone(),
            app.state(),
            app_handle.clone(),
        )
        .await
        .unwrap();
        let msg_text = "Hello via P2P!";
        let result = send_message::<tauri::test::MockRuntime>(
            local_peer_id.clone(),
            msg_text.to_string(),
            app.state(),
            app_handle.clone(),
        )
        .await;
        assert!(result.is_ok());
        let sent_msg = result.unwrap();
        assert_eq!(sent_msg.body, msg_text);
        assert_eq!(sent_msg.user_id, 0);

        let data_dir = app.app_handle().path().app_data_dir().unwrap();
        let profile_data = auth::load(username, &data_dir).await.unwrap();
        let chat = profile_data
            .chats
            .iter()
            .find(|c| c.peer_id == local_peer_id)
            .unwrap();
        assert_eq!(chat.messages.len() > 0, true);
    }

    #[tokio::test]
    async fn test_get_chats_returns_list_of_chats_with_unread_flag() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "chatter";
        let password = "pass";
        let data_dir = app.app_handle().path().app_data_dir().unwrap();
        cleanup_profile(username, &data_dir);
        let result =
            login::<tauri::test::MockRuntime>(username, password, app.state(), app_handle.clone())
                .await
                .unwrap();
        let mut profile_data = auth::load(username, &data_dir).await.unwrap();
        let msg = auth::Message {
            username: "other-chatter-2".to_string(),
            text: "Hello from other chatter!".to_string(),
            created_at: chrono::Local::now().to_rfc3339(),
            received_at: "".into(),
        };
        profile_data.chats.push(auth::Chat {
            name: "other-chatter-2".to_string(),
            peer_id: "other-chatter-2".to_string(),
            messages: vec![msg],
        });
        auth::save(profile_data, &data_dir).await.unwrap();
        let chats = get_chats::<tauri::test::MockRuntime>(app.state(), app_handle.clone())
            .await
            .unwrap();
        let found = chats.iter().find(|c| c.login == "other-chatter-2");
        assert!(found.is_some());
        assert!(found.unwrap().has_unread);
    }

    #[tokio::test]
    async fn test_get_chat_returns_paginated_messages() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "chatter";
        let password = "pass";
        let data_dir = app.app_handle().path().app_data_dir().unwrap();
        cleanup_profile(username, &data_dir);
        let result =
            login::<tauri::test::MockRuntime>(username, password, app.state(), app_handle.clone())
                .await
                .unwrap();
        let mut profile_data = auth::load(username, &data_dir).await.unwrap();
        let mut messages = Vec::new();
        for i in 0..10 {
            messages.push(auth::Message {
                username: if i % 2 == 0 {
                    "chatter"
                } else {
                    "other-chatter"
                }
                .to_string(),
                text: format!("Message #{}", i),
                created_at: chrono::Local::now().to_rfc3339(),
                received_at: chrono::Local::now().to_rfc3339(),
            });
        }
        profile_data.chats.push(auth::Chat {
            name: "chatter".to_string(),
            peer_id: "chatter".to_string(),
            messages,
        });
        auth::save(profile_data, &data_dir).await.unwrap();

        let result = get_chat::<tauri::test::MockRuntime>(
            "chatter".into(),
            1,
            2,
            app.state(),
            app_handle.clone(),
        )
        .await
        .unwrap();

        assert_eq!(result.meta.page, 1);
        assert_eq!(result.meta.per_page, 2);
        assert_eq!(result.meta.total, 10);
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].body, "Message #9");
    }

    #[tokio::test]
    async fn test_login_returns_user_and_saves_profile_on_first_login() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "newuser";
        let password = "pass123";
        let result =
            login::<tauri::test::MockRuntime>(username, password, app.state(), app_handle.clone())
                .await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.login, username);

        let data_dir = app.app_handle().path().app_data_dir().unwrap();
        let loaded = auth::load(username, &data_dir).await.unwrap();
        assert_eq!(loaded.username, username);
    }

    #[tokio::test]
    async fn test_get_user_returns_current_user() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let username = "current";
        let _ =
            login::<tauri::test::MockRuntime>(username, "pass", app.state(), app_handle.clone())
                .await
                .unwrap();
        let user = get_user::<tauri::test::MockRuntime>(app.state(), app_handle.clone())
            .await
            .unwrap();
        assert_eq!(user.login, username);
    }

    #[tokio::test]
    async fn test_handle_incoming_message_saves_and_emits() {
        // let temp_dir = tempfile::tempdir().unwrap();
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let app_handle = app.handle();
        let data_dir = app_handle.path().app_data_dir().unwrap();
        let username = "test_user_1";

        let profile = Profile::new(username.to_string(), "pass".to_string()).unwrap();
        let device_id = profile.device_id.clone();
        let state = app.state::<AppState>();
        {
            let mut lock = state.cur_profile.lock().unwrap();
            *lock = Some(profile);
        }

        let mut profile_data = ProfileData::default();
        profile_data.username = username.to_string();
        auth::save(profile_data, &data_dir).await.unwrap();

        let keypair = get_peer_id_from_device_id(&device_id).unwrap();
        let mut msg = ChatMessage::new("test_user_2".to_string(), "Hello!".to_string());
        msg.sender_public_key = keypair.public().encode_protobuf();
        let signed_msg = msg.sign(&keypair);
        let result = handle_incoming_message(app_handle.clone(), signed_msg).await;
        assert!(result.is_ok());

        let loaded = auth::load(username, &data_dir).await.unwrap();
        let test_user_2_chat = loaded
            .chats
            .iter()
            .find(|c| c.name == "test_user_2")
            .unwrap();
        assert_eq!(test_user_2_chat.messages.len(), 1);
        assert_eq!(test_user_2_chat.messages[0].text, "Hello!");
    }

    #[tokio::test]
    async fn incoming_message_is_verified_and_saved_to_profile() {
        let temp_dir = tempfile::tempdir().unwrap();
        let username = "test_user";
        let device_id = "0000000000000000000000000000000000000000000000000000000000000001";

        let mut profile_data = ProfileData::default();
        profile_data.username = username.to_string();
        auth::save(profile_data, temp_dir.path()).await.unwrap();

        let kp = p2p::identity::get_peer_id_from_device_id(device_id).unwrap();
        let incoming_msg = crate::p2p::message::ChatMessage::new(
            "peer123".to_string(),
            "Hello from test!".to_string(),
        )
        .sign(&kp);
        let result =
            process_and_save_incoming_message(&incoming_msg, username, temp_dir.path()).await;
        assert!(result.is_ok());
        let saved_data = auth::load(username, temp_dir.path()).await.unwrap();
        assert_eq!(saved_data.chats.len(), 1);
        assert_eq!(saved_data.chats[0].name, "peer123");
    }

    #[tokio::test]
    async fn incoming_message_with_invalid_signature_is_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let username = "test_user";
        let device_id = "0000000000000000000000000000000000000000000000000000000000000001";

        let mut profile_data = ProfileData::default();
        profile_data.username = username.to_string();
        auth::save(profile_data, temp_dir.path()).await.unwrap();

        let mut fake_msg =
            crate::p2p::message::ChatMessage::new("attacker".to_string(), "Hacked!".to_string());
        fake_msg.signature = vec![1, 2, 3];
        let result = process_and_save_incoming_message(&fake_msg, username, temp_dir.path()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid signature"));
    }

    #[test]
    fn get_local_peer_iid_returns_when_initialized() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let device_id = "0000000000000000000000000000000000000000000000000000000000000001";
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = init_p2p::<tauri::test::MockRuntime>(
                device_id.to_string(),
                app.app_handle().clone(),
            )
            .await;
        });
        let result = get_local_peer_id(app.state::<AppState>());
        assert!(result.is_ok());
        let peer_id_str = result.unwrap();
        assert!(!peer_id_str.is_empty());
    }

    #[test]
    fn get_local_peer_id_returns_error_when_not_initialized() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let result = get_local_peer_id(app.state::<AppState>());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[tokio::test]
    async fn send_p2p_message_returns_ok_for_valid_input() {
        let app = tauri::test::mock_builder()
            .manage(AppState::default())
            .build(mock_context(noop_assets()))
            .expect("Failed to build mock app");
        let device_id = "0000000000000000000000000000000000000000000000000000000000000001";
        let _ =
            init_p2p::<tauri::test::MockRuntime>(device_id.to_string(), app.app_handle().clone())
                .await;
        let state = app.state::<AppState>();
        let profile_kp = p2p::identity::get_peer_id_from_device_id(device_id).unwrap();
        let peer_id = PeerId::from(profile_kp.public());

        let result = send_p2p_message::<tauri::test::MockRuntime>(
            peer_id.to_string(),
            "Hello P2P!".to_string(),
            app.app_handle().clone(),
        )
        .await;
        assert!(result.is_ok());
    }

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

    fn cleanup_profile(username: &str, data_dir: &Path) {
        let file_path = data_dir.join(format!("{}.json", username));
        let _ = std::fs::remove_file(file_path);
    }
}
