use crate::{Behaviour, BehaviourEvent};
use futures::channel::mpsc;
use futures::future::Either;
use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{ping, Multiaddr, Swarm};

pub async fn run(mut swarm: Swarm<Behaviour>, mut commands: mpsc::Receiver<Command>) {
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
            Either::Left((_, _)) => {}
            Either::Right((Some(Command::Dial(addr)), _)) => {
                let _ = swarm.dial(addr);
            }
            Either::Right((None, _)) => {
                log::info!("Sender dropped, terminating event loop");
                return;
            }
        }
    }
}

pub enum Command {
    Dial(Multiaddr),
}
