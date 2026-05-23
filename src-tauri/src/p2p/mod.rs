use libp2p::PeerId;

// pub mod chat;
pub use crate::p2p::message::ChatMessage;
pub mod error;
pub mod identity;
pub mod message;
pub mod swarm;

pub enum P2pCommand {
    SendMessage {
        peer_id: PeerId,
        message: ChatMessage,
    },
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::p2p::message::ChatMessage;
    use crate::p2p::swarm::{ChatBehaviour, ChatBehaviourEvent, RelayChatBehaviourEvent, create_swarm_with_rx, run_swarm_loop};
    use crate::p2p::{error::P2pError, identity::get_peer_id_from_device_id, swarm::create_swarm};

    use futures::StreamExt;
    use libp2p::core::transport::MemoryTransport;
    use libp2p::multiaddr::{Multiaddr, Protocol};
    use libp2p::multihash::Multihash;
    use libp2p::{relay, request_response};
    use libp2p::{
        core::upgrade,
        identity::Keypair,
        noise, ping,
        swarm::{behaviour, NetworkBehaviour, SwarmBuilder, SwarmEvent},
        tcp, yamux, PeerId, Swarm, Transport,
    };

    #[test]
    fn test_signature_verification_passes_on_valid() {
       let device_id = "0000000000000000000000000000000000000000000000000000000000000001";
       let kp = get_peer_id_from_device_id(device_id).unwrap();
       let msg = ChatMessage::new(
           "test".into(),
           "Test Signed!".into(),
       );
       let signed = msg.sign(&kp);
       assert!(signed.verify_signature().is_ok());
       assert_eq!(signed.content, "Test Signed!");
    }
    #[test]
    fn test_signature_verification_fails_on_tampered() {
       let device_id = "0000000000000000000000000000000000000000000000000000000000000001";
       let kp = get_peer_id_from_device_id(device_id).unwrap();
       let mut msg = ChatMessage::new(
           "test".into(),
           "Original".into(),
       );
       let signed = msg.sign(&kp);
       let mut tampered = signed.clone();
       tampered.content = "Hacked!".into();
       assert!(tampered.verify_signature().is_err());
    }

    #[tokio::test]
    async fn test_connection_via_relay_fallback() {
        use std::time::Duration;
        use libp2p::swarm::SwarmEvent;
        use crate::p2p::swarm::{create_relay_server, create_swarm_with_relay, RelayChatBehaviourEvent};

        let relay_kp = Keypair::generate_ed25519();
        let mut relay_swarm = create_relay_server(relay_kp).await.unwrap();
        let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        relay_swarm.listen_on(listen_addr.clone()).unwrap();
        
        let relay_addr = loop {
            if let SwarmEvent::NewListenAddr { address, .. } = relay_swarm.select_next_some().await {
                break address;
            }
        };
        let relay_peer_id = *relay_swarm.local_peer_id();

        let client1_kp = Keypair::generate_ed25519();
        let client2_kp = Keypair::generate_ed25519();
        let mut client1 = create_swarm_with_relay(client1_kp).await.unwrap();
        let mut client2 = create_swarm_with_relay(client2_kp).await.unwrap();

        let client1_id = *client1.local_peer_id();
        let client2_id = *client2.local_peer_id();

        let relay_multiaddr = relay_addr.with(Protocol::P2p(relay_peer_id.into()));
        client1.dial(relay_multiaddr.clone()).unwrap();
        client2.dial(relay_multiaddr).unwrap();

        tokio::time::timeout(Duration::from_secs(10), async {
            let mut client1_connected = false;
            let mut client2_connected = false;

            loop {
                tokio::select! {
                    event = client1.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == relay_peer_id {
                                client1_connected = true;
                            }
                        }
                    }
                    event = client2.select_next_some() => {
                        if let SwarmEvent::ConnectionEstablished { peer_id, .. } = event {
                            if peer_id == relay_peer_id {
                                client2_connected = true;
                            }
                        }
                    }
                    _ = relay_swarm.select_next_some() => {}
                }
                if client1_connected && client2_connected { break; }
            }
        }).await.expect("Timeout: relay reservation failed");

    }

    #[tokio::test]
    async fn test_message_stream_emits_exactly_once() {
        use tokio::sync::mpsc;
       let id1 = "0000000000000000000000000000000000000000000000000000000000000001";
       let id2 = "0000000000000000000000000000000000000000000000000000000000000002";
       let kp1 = get_peer_id_from_device_id(id1).unwrap();
       let kp2 = get_peer_id_from_device_id(id2).unwrap();

       let (mut swarm1,_, _, _, mut rx1) = create_swarm_with_rx(kp1).await.unwrap();
       let (mut swarm2, _, _, tx2, mut rx2) = create_swarm_with_rx(kp2).await.unwrap();
       connect_swarms(&mut swarm1, &mut swarm2).await;
       let msg = ChatMessage::new("peer1".into(),"Hello P2P!".into());
       let peer2 = *swarm2.local_peer_id();
       swarm1.behaviour_mut().messages.send_request(&peer2, msg.clone());

       tokio::time::timeout(Duration::from_secs(5), async {
           loop {
               tokio::select! {
                   _ = swarm1.select_next_some() => {},
                   event = swarm2.select_next_some() => {
                       if let SwarmEvent::Behaviour(
                           ChatBehaviourEvent::Messages(
                            request_response::Event::Message { 
                                message: request_response::Message::Request { request, channel, .. },
                                ..
                                }
                               )
                           ) = event {
                           let _ = swarm2.behaviour_mut().messages.send_response(channel, request.clone());
                           let _ = tx2.send(request);
                           break;
                       }
                   }
               }
           }
       })
       .await
       .expect("timeout waiting msg from channel");

       let received = rx2.try_recv().expect("channel must receive msg");

        assert_eq!(received.content, "Hello P2P!");
        assert!(rx2.try_recv().is_err(), "msg must received only once time!");
    }

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
        let msg = ChatMessage::new( "peer1".into(), "Hello P2P!".into() );
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
