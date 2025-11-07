//! Network layer implementation with Dandelion++ and Tor support

mod p2p;
mod dandelion;
mod tor;

pub use p2p::*;
pub use dandelion::*;
pub use tor::*;

use crate::types::{Transaction, Block};
use libp2p::{
    core::upgrade,
    identity,
    noise,
    tcp::TokioTcpConfig,
    yamux,
    NetworkBehaviour,
    PeerId,
    Transport,
};
use std::error::Error;
use tokio::sync::mpsc;

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Enable Tor SOCKS5 proxy
    pub use_tor: bool,
    /// Tor SOCKS5 proxy address
    pub tor_proxy: Option<String>,
    /// Listen addresses
    pub listen_addresses: Vec<String>,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Enable Dandelion++
    pub use_dandelion: bool,
}