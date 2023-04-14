use clap::Parser;
use env_logger::Env;
use futures::stream::StreamExt;
use libp2p::{
    core::upgrade::Version, identity, noise, ping, swarm::SwarmBuilder, swarm::SwarmEvent, tcp,
    yamux, Multiaddr, Transport,
};
use std::error::Error;

#[derive(Debug, Parser)]
#[clap(name = "libp2p-workshop-node")]
struct Opts {
    #[clap(long)]
    bootstrap_node: Multiaddr,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let opts = Opts::parse();

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    log::info!("Local peer id: {local_peer_id}");

    let transport = tcp::async_io::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise::NoiseAuthenticated::xx(&local_key)?)
        .multiplex(yamux::YamuxConfig::default())
        .boxed();

    let mut swarm = SwarmBuilder::with_async_std_executor(
        transport,
        ping::Behaviour::new(ping::Config::default()),
        local_peer_id,
    )
    .build();

    swarm.dial(opts.bootstrap_node)?;

    loop {
        match swarm.next().await.unwrap() {
            SwarmEvent::ConnectionEstablished { endpoint, .. } => {
                log::info!("New connection to {}.", endpoint.get_remote_address());
            }
            SwarmEvent::ConnectionClosed { endpoint, .. } => {
                log::info!("Closed connection to {}.", endpoint.get_remote_address());
            }
            SwarmEvent::Behaviour(ping::Event {
                peer,
                result: Ok(ping::Success::Ping { rtt }),
            }) => {
                log::info!("RTT tp {peer} is {}s", rtt.as_secs());
            }
            e => {
                log::debug!("{:?}", e)
            }
        }
    }
}
