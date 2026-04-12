#![allow(unused)]
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    identity::Keypair,
    noise, ping,
    request_response::{self, ProtocolSupport},
    swarm::{behaviour, NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Swarm, Transport,
};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::p2p::{
    error::P2pError,
    identity,
    message::{ChatMessage, ChatProtocol, JsonCodec},
};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ChatBehaviourEvent")]
pub struct ChatBehaviour {
    ping: ping::Behaviour,
    pub messages: request_response::Behaviour<JsonCodec>,
}
impl Default for ChatBehaviour {
    fn default() -> Self {
        Self {
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(2))),
            messages: request_response::Behaviour::new(
                JsonCodec,
                [(ChatProtocol, ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        }
    }
}

pub async fn create_swarm(keypair: Keypair) -> Result<Swarm<ChatBehaviour>, P2pError> {
    let peer_id = PeerId::from(keypair.public());
    let behaviour = ChatBehaviour::default();

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair).expect("Noise config"))
        .multiplex(yamux::Config::default())
        .boxed();

    let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    Ok(swarm)
}

pub async fn create_swarm_with_rx(
    keypair: Keypair,
) -> Result<
    (
        Swarm<ChatBehaviour>,
        mpsc::UnboundedSender<ChatMessage>,
        mpsc::UnboundedReceiver<ChatMessage>,
    ),
    P2pError,
> {
    let swarm = create_swarm(keypair).await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel();
    Ok((swarm, tx, rx))
}

pub async fn run_swarm_loop(
    mut swarm: Swarm<ChatBehaviour>,
    tx: mpsc::UnboundedSender<ChatMessage>,
) {
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(ChatBehaviourEvent::Messages(
                request_response::Event::Message {
                    message:
                        request_response::Message::Request {
                            request, channel, ..
                        },
                    ..
                },
            )) => {
                // Автоматический ACK (RequestResponse требует ответа)
                let _ = swarm
                    .behaviour_mut()
                    .messages
                    .send_response(channel, request.clone());
                let _ = tx.send(request);
            }
            SwarmEvent::ConnectionClosed { .. } | SwarmEvent::IncomingConnectionError { .. } => {
                break;
            }
            _ => {} // ping, identify
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::core::transport::MemoryTransport;
    use libp2p::multiaddr::{Multiaddr, Protocol};
    use libp2p::multihash::Multihash;

    use crate::p2p::identity::get_peer_id_from_device_id;

    use super::*;
}
