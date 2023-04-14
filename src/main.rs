mod event_loop;

use crate::event_loop::Command;
use async_std::io;
use async_std::io::prelude::BufReadExt;
use async_std::io::BufReader;
use env_logger::Env;
use futures::channel::mpsc;
use futures::{SinkExt, TryStreamExt};
use libp2p::swarm::keep_alive;
use libp2p::{
    core::upgrade::Version, dns, gossipsub, identify, identity, noise, ping, swarm::SwarmBuilder,
    tcp, yamux, Transport,
};

const BOOTSTRAP_NODE: &str = "/dns4/libp2p-workshop-bootnode.fly.dev/tcp/9999";

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    log::info!("Local peer id: {local_peer_id}");

    let transport = dns::DnsConfig::system(tcp::async_io::Transport::default())
        .await?
        .upgrade(Version::V1Lazy)
        .authenticate(noise::NoiseAuthenticated::xx(&local_key)?)
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    let swarm =
        SwarmBuilder::with_async_std_executor(transport, Behaviour::new(local_key), local_peer_id)
            .build();

    let (mut sender, receiver) = mpsc::channel(1);

    async_std::task::spawn(event_loop::run(swarm, receiver));

    sender.send(Command::Dial(BOOTSTRAP_NODE.parse()?)).await?;

    let mut reader = BufReader::new(io::stdin()).lines();

    while let Some(line) = reader.try_next().await? {
        match line
            .split(":")
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["send", message] => {
                sender.send(Command::Publish((*message).to_owned())).await?;
            }
            ["dial", address] => {
                sender.send(Command::Dial(address.parse()?)).await?;
            }
            ["quit"] => {
                break;
            }
            [other, ..] => {
                log::warn!("Unknown command: {other}");
            }
            [] => {}
        }
    }

    Ok(())
}

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct Behaviour {
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
    identify: identify::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

impl Behaviour {
    fn new(local_key: identity::Keypair) -> Self {
        Self {
            ping: ping::Behaviour::default(),
            keep_alive: keep_alive::Behaviour,
            identify: identify::Behaviour::new(identify::Config::new(
                "/libp2p-workshop/1.0.0".into(),
                local_key.public(),
            )),
            gossipsub: gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(local_key),
                gossipsub::Config::default(),
            )
            .unwrap(),
        }
    }
}
