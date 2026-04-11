// pub mod chat;
pub mod error;
pub mod identity;
pub mod message;
pub mod swarm;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p::message::ChatMessage;
    use crate::p2p::swarm::{ChatBehaviour, ChatBehaviourEvent};
    use crate::p2p::{error::P2pError, identity::get_peer_id_from_device_id, swarm::create_swarm};

    use futures::StreamExt;
    use libp2p::core::transport::MemoryTransport;
    use libp2p::multiaddr::{Multiaddr, Protocol};
    use libp2p::multihash::Multihash;
    use libp2p::request_response;
    use libp2p::{
        core::upgrade,
        identity::Keypair,
        noise, ping,
        swarm::{behaviour, NetworkBehaviour, SwarmBuilder, SwarmEvent},
        tcp, yamux, PeerId, Swarm, Transport,
    };

    async fn connect_swarms(swarm1: &mut Swarm<ChatBehaviour>, swarm2: &mut Swarm<ChatBehaviour>) {
        let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        swarm2.listen_on(listen_addr).unwrap();

        let addr = loop {
            if let SwarmEvent::NewListenAddr { address, .. } = swarm2.select_next_some().await {
                break address;
            }
        };

        let peer2 = *swarm2.local_peer_id();
        let target = addr.with(Protocol::P2p(
            Multihash::from_bytes(&peer2.to_bytes()).unwrap(),
        ));
        swarm1.dial(target).unwrap();

        tokio::time::timeout(Duration::from_secs(5), async {
            let mut c1 = false;
            let mut c2 = false;
            loop {
                tokio::select! {
                    event = swarm1.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == peer2 {
                                c1 = true;
                            }
                        }
                    }
                    event = swarm2.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == *swarm1.local_peer_id() {
                                c2 = true;
                            }
                        }
                    }
                }
                if c1 && c2 {
                    break;
                }
            }
        })
        .await
        .expect("Connection establish timeout");
    }

    #[tokio::test]
    async fn test_send_and_receive_message() {
       let id1 = "0000000000000000000000000000000000000000000000000000000000000001";
        let id2 = "0000000000000000000000000000000000000000000000000000000000000002";
        let kp1 = get_peer_id_from_device_id(id1).unwrap();
        let kp2 = get_peer_id_from_device_id(id2).unwrap();

        let mut swarm1 = create_swarm(kp1).await.unwrap();
        let mut swarm2 = create_swarm(kp2).await.unwrap();

        let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        swarm2.listen_on(listen_addr.clone()).unwrap();
        let addr = loop {
            if let SwarmEvent::NewListenAddr { address, .. } = swarm2.select_next_some().await { break address; }
        };
        let peer2 = *swarm2.local_peer_id();
        let target = addr.with(Protocol::P2p(Multihash::from_bytes(&peer2.to_bytes()).unwrap()));
        swarm1.dial(target).unwrap();

        tokio::time::timeout(Duration::from_secs(5), async {
            let mut c1 = false; let mut c2 = false;
            loop {
                tokio::select! {
                    event = swarm1.select_next_some() => { if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event { if peer_id == peer2 { c1 = true; } } }
                    event = swarm2.select_next_some() => { if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event { if peer_id == *swarm1.local_peer_id() { c2 = true; } } }
                }
                if c1 && c2 { break; }
            }
        }).await.unwrap();

        // Отправляем сообщение
        let msg = ChatMessage { sender: "peer1".into(), content: "Hello P2P!".into() };
        let _request_id = swarm1.behaviour_mut().messages.send_request(&peer2, msg.clone());

        // Ожидаем получение и отвечаем
        let received = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                tokio::select! {
                    _event = swarm1.select_next_some() => {}
                    event = swarm2.select_next_some() => {
                        if let SwarmEvent::Behaviour(ChatBehaviourEvent::Messages(
                            request_response::Event::Message { 
                                message: request_response::Message::Request { request, channel, .. }, 
                                .. 
                            }
                        )) = event {
                            // Отвечаем, чтобы запрос завершился
                            swarm2.behaviour_mut().messages.send_response(channel, request.clone()).unwrap();
                            return request;
                        }
                    }
                }
            }
        }).await.unwrap();

        assert_eq!(received.content, "Hello P2P!"); 
    }

    #[tokio::test]
    async fn test_establish_connection() {
        let id1 = "0000000000000000000000000000000000000000000000000000000000000001";
        let id2 = "0000000000000000000000000000000000000000000000000000000000000002";
        let kp1 = get_peer_id_from_device_id(id1).unwrap();
        let kp2 = get_peer_id_from_device_id(id2).unwrap();

        let mut swarm1 = create_swarm(kp1).await.unwrap();
        let mut swarm2 = create_swarm(kp2).await.unwrap();
        connect_swarms(&mut swarm1, &mut swarm2).await;
    }
}
