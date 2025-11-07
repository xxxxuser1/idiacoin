//! Core P2P networking implementation

use super::*;
use libp2p::{
    gossipsub::{
        Gossipsub, GossipsubConfig, GossipsubConfigBuilder,
        MessageAuthenticity, ValidationMode,
    },
    swarm::SwarmBuilder,
    Multiaddr,
    Swarm,
};
use std::time::Duration;

/// P2P network events
#[derive(Debug)]
pub enum NetworkEvent {
    /// New transaction received
    Transaction(Transaction),
    /// New block received
    Block(Block),
    /// New peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
}

/// P2P network service
pub struct P2PService {
    /// libp2p swarm
    swarm: Swarm<IdiaNetworkBehaviour>,
    /// Event channel sender
    event_sender: mpsc::Sender<NetworkEvent>,
    /// Event channel receiver
    event_receiver: mpsc::Receiver<NetworkEvent>,
}

/// Custom network behaviour
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NetworkEvent")]
pub struct IdiaNetworkBehaviour {
    /// Gossipsub for p2p message propagation
    gossipsub: Gossipsub,
}

impl P2PService {
    /// Create a new P2P service
    pub async fn new(config: NetworkConfig) -> Result<Self, Box<dyn Error>> {
        // Generate key pair
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());

        // Set up gossipsub
        let gossipsub_config = GossipsubConfigBuilder::default()
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|message| {
                // Custom message ID function
                let mut hasher = Sha256::new();
                hasher.update(message.data.as_slice());
                hasher.finalize().into()
            })
            .build()
            .expect("Valid gossipsub config");

        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )?;

        // Create transport
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        // Create swarm
        let behaviour = IdiaNetworkBehaviour {
            gossipsub,
        };

        let mut swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        // Listen on addresses
        for addr in config.listen_addresses {
            swarm.listen_on(addr.parse()?)?;
        }

        // Create event channels
        let (tx, rx) = mpsc::channel(100);

        Ok(Self {
            swarm,
            event_sender: tx,
            event_receiver: rx,
        })
    }

    /// Start the P2P service
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                event = self.swarm.next() => {
                    if let Some(event) = event {
                        self.handle_swarm_event(event).await;
                    }
                }
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    // Periodic maintenance
                    self.maintain().await;
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::Transaction(tx) => {
                // Handle new transaction
                if let Err(e) = self.event_sender.send(NetworkEvent::Transaction(tx)).await {
                    log::error!("Failed to send transaction event: {}", e);
                }
            }
            NetworkEvent::Block(block) => {
                // Handle new block
                if let Err(e) = self.event_sender.send(NetworkEvent::Block(block)).await {
                    log::error!("Failed to send block event: {}", e);
                }
            }
            NetworkEvent::PeerConnected(peer_id) => {
                log::info!("Peer connected: {}", peer_id);
            }
            NetworkEvent::PeerDisconnected(peer_id) => {
                log::info!("Peer disconnected: {}", peer_id);
            }
        }
    }

    /// Periodic maintenance
    async fn maintain(&mut self) {
        // Cleanup, reconnect to peers, etc.
    }

    /// Broadcast a transaction to the network
    pub async fn broadcast_transaction(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
        let encoded = bincode::serialize(&tx)?;
        self.swarm.behaviour_mut().gossipsub.publish(
            "transactions".into(),
            encoded,
        )?;
        Ok(())
    }

    /// Broadcast a block to the network
    pub async fn broadcast_block(&mut self, block: Block) -> Result<(), Box<dyn Error>> {
        let encoded = bincode::serialize(&block)?;
        self.swarm.behaviour_mut().gossipsub.publish(
            "blocks".into(),
            encoded,
        )?;
        Ok(())
    }
}