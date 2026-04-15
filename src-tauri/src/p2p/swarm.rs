#![allow(unused)]
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    dcutr, identify,
    identity::Keypair,
    noise, ping, relay,
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
#[behaviour(to_swarm = "RelayChatEvent")]
pub struct RelayChatBehaviour {
    pub base: ChatBehaviour,
    pub relay_client: relay::client::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub identify: identify::Behaviour,
}

pub async fn create_swarm_with_relay(
    keypair: Keypair,
) -> Result<Swarm<RelayChatBehaviour>, P2pError> {
    let peer_id = PeerId::from(keypair.public());

    let (relay_transport, relay_client) = relay::client::new(peer_id);

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .or_transport(relay_transport)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair).unwrap())
        .multiplex(yamux::Config::default())
        .boxed(); // Now the types match perfectly

    let behaviour = RelayChatBehaviour {
        base: ChatBehaviour::default(),
        relay_client,
        dcutr: dcutr::Behaviour::new(peer_id),
        identify: identify::Behaviour::new(
            identify::Config::new("/meshee/1.0".into(), keypair.public())
                .with_push_listen_addr_updates(true),
        ),
    };

    Ok(SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build())
}

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

#[derive(NetworkBehaviour)]
pub struct TestRelayServer {
    pub relay: relay::Behaviour,
    pub identify: identify::Behaviour,
}

pub async fn create_relay_server(keypair: Keypair) -> Result<Swarm<TestRelayServer>, P2pError> {
    let peer_id = PeerId::from(keypair.public());
    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair).expect("Noise"))
        .multiplex(yamux::Config::default())
        .boxed();
    let behaviour = TestRelayServer {
        relay: relay::Behaviour::new(peer_id, relay::Config::default()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/meshee-relay/1.0".into(),
            keypair.public(),
        )),
    };
    Ok(SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build())
}

#[cfg(test)]
mod tests {
    use libp2p::core::transport::MemoryTransport;
    use libp2p::multiaddr::{Multiaddr, Protocol};
    use libp2p::multihash::Multihash;

    use crate::p2p::identity::get_peer_id_from_device_id;

    use super::*;
}
