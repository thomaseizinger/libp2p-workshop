use env_logger::Env;
use futures::stream::StreamExt;
use libp2p::swarm::keep_alive;
use libp2p::{
    core::upgrade::Version, dns, identify, identity, noise, ping, swarm::SwarmBuilder,
    swarm::SwarmEvent, tcp, yamux, Multiaddr, Transport,
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

    let mut swarm = SwarmBuilder::with_async_std_executor(
        transport,
        Behaviour::new(local_key.public()),
        local_peer_id,
    )
    .build();

    swarm.dial(BOOTSTRAP_NODE.parse::<Multiaddr>()?)?;

    loop {
        match swarm.next().await.unwrap() {
            SwarmEvent::ConnectionEstablished { endpoint, .. } => {
                log::info!("New connection to {}", endpoint.get_remote_address());
            }
            SwarmEvent::ConnectionClosed { endpoint, .. } => {
                log::info!("Closed connection to {}", endpoint.get_remote_address());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(ping::Success::Ping { rtt }),
            })) => {
                log::info!("RTT to {peer} is {}ms", rtt.as_millis());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info,
            })) => {
                log::info!(
                    "Peer {peer_id} supports protocols: {}",
                    info.protocols.join(",")
                );
            }
            e => {
                log::debug!("{:?}", e)
            }
        }
    }
}

#[derive(libp2p::swarm::NetworkBehaviour)]
struct Behaviour {
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
    identify: identify::Behaviour,
}

impl Behaviour {
    fn new(local_key: identity::PublicKey) -> Self {
        Self {
            ping: ping::Behaviour::default(),
            keep_alive: keep_alive::Behaviour,
            identify: identify::Behaviour::new(identify::Config::new(
                "/libp2p-workshop/1.0.0".into(),
                local_key,
            )),
        }
    }
}
