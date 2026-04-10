#![allow(unused)]
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    identity::Keypair,
    noise, ping,
    swarm::{behaviour, NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Swarm, Transport,
};
use std::time::Duration;

use crate::p2p::{error::P2pError, identity};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ChatBehaviourEvent")]
pub struct ChatBehaviour {
    ping: ping::Behaviour,
}
impl Default for ChatBehaviour {
    fn default() -> Self {
        Self {
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(2))),
        }
    }
}

pub async fn create_swarm(device_id: &str) -> Result<Swarm<ChatBehaviour>, P2pError> {
    let keypair = identity::get_peer_id_from_device_id(device_id)?;
    let peer_id = PeerId::from(keypair.public());
    let behaviour = ChatBehaviour::default();

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair).expect("Noise config"))
        .multiplex(yamux::Config::default())
        .boxed();

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair).expect("Noise config"))
        .multiplex(yamux::Config::default())
        .boxed();

    let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use libp2p::core::transport::MemoryTransport;
    use libp2p::multiaddr::{Multiaddr, Protocol};
    use libp2p::multihash::Multihash;

    use super::*;

    #[tokio::test]
    async fn test_establish_connection() {
        let first_device_id = "0000000000000000000000000000000000000000000000000000000000000001";
        let second_device_id = "0000000000000000000000000000000000000000000000000000000000000002";
        let mut first_swarm = create_swarm(first_device_id).await.unwrap();
        let mut second_swarm = create_swarm(second_device_id).await.unwrap();
        let second_addr = "/ip4/127.0.0.1/tcp/0".parse::<Multiaddr>().unwrap();
        second_swarm.listen_on(second_addr).unwrap();
        let second_addr = loop {
            if let SwarmEvent::NewListenAddr { address, .. } = second_swarm.select_next_some().await
            {
                break address;
            }
        };
        let second_peer_id = second_swarm.local_peer_id().clone();
        let second_addr_with_peer = second_addr.with(Protocol::P2p(
            Multihash::from_bytes(&second_peer_id.to_bytes()).unwrap(),
        ));
        first_swarm.dial(second_addr_with_peer).unwrap();

        tokio::time::timeout(Duration::from_secs(5), async {
            let mut first_connected = false;
            let mut second_connected = false;
            loop {
                tokio::select! {
                    event = first_swarm.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == second_peer_id {
                                first_connected = true;
                            }
                        }
                    }
                    event = second_swarm.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == *first_swarm.local_peer_id() {
                                second_connected = true;
                            }
                        }
                    }
                }
                if first_connected && second_connected {
                    break;
                }
            }
        })
        .await
        .expect("Connection establish timeout");
    }
}
