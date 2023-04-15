use crate::{Behaviour, BehaviourEvent};
use futures::channel::mpsc;
use futures::future::Either;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{gossipsub, ping, Multiaddr, Swarm};

const CHAT_TOPIC: &'static str = "workshop-chat";

pub async fn run(mut swarm: Swarm<Behaviour>, mut commands: mpsc::Receiver<Command>) {
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new(CHAT_TOPIC))
        .unwrap();
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&gossipsub::IdentTopic::new("universal-connectivity"))
        .unwrap();

    loop {
        match futures::future::select(swarm.select_next_some(), commands.next()).await {
            Either::Left((SwarmEvent::ConnectionEstablished { endpoint, .. }, _)) => {
                log::info!("Connected to {}", endpoint.get_remote_address());
            }
            Either::Left((
                SwarmEvent::ConnectionClosed {
                    endpoint,
                    num_established,
                    ..
                },
                _,
            )) if num_established == 0 => {
                log::info!("Disconnected from {}", endpoint.get_remote_address());
            }
            Either::Left((SwarmEvent::NewListenAddr { address, .. }, _)) => {
                log::info!("Listening on {address}");
            }
            Either::Left((
                SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
                    peer,
                    result: Ok(ping::Success::Ping { rtt }),
                })),
                _,
            )) => {
                log::info!("RTT tp {peer} is {}ms", rtt.as_millis());
            }
            Either::Left((
                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    message,
                    ..
                })),
                _,
            )) => {
                let source = message
                    .source
                    .map(|p| p.to_string())
                    .unwrap_or("unknown".to_owned());

                log::info!(
                    "Message from {}: {}",
                    source,
                    String::from_utf8_lossy(&message.data)
                );
            }
            Either::Left((_, _)) => {}
            Either::Right((Some(Command::Dial(addr)), _)) => {
                let _ = swarm.dial(addr);
            }
            Either::Right((Some(Command::Publish(message)), _)) => {
                let _ = swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(gossipsub::IdentTopic::new(CHAT_TOPIC), message.into_bytes());
            }
            Either::Right((None, _)) => {
                log::info!("Sender dropped, terminating event loop");
                return;
            }
        }
    }
}

pub enum Command {
    Publish(String),
    Dial(Multiaddr),
}
