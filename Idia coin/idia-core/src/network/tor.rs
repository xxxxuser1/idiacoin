//! Tor network integration

use super::*;
use tor_client::{TorClient, TorClientConfig};
use std::net::SocketAddr;

/// Tor network handler
pub struct TorHandler {
    /// Tor client
    client: TorClient,
    /// SOCKS5 proxy address
    proxy_addr: SocketAddr,
}

impl TorHandler {
    /// Create a new Tor handler
    pub async fn new(proxy_addr: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let config = TorClientConfig::default();
        let client = TorClient::create(config).await?;

        Ok(Self {
            client,
            proxy_addr,
        })
    }

    /// Create a new connection through Tor
    pub async fn connect(&self, address: &str) -> Result<tokio::net::TcpStream, Box<dyn Error>> {
        self.client.connect(address).await.map_err(Into::into)
    }

    /// Get the SOCKS5 proxy address
    pub fn proxy_addr(&self) -> SocketAddr {
        self.proxy_addr
    }

    /// Check if Tor is ready
    pub async fn check_tor(&self) -> bool {
        self.client.check_connectivity().await.is_ok()
    }
}

/// Extension trait for network config
pub trait TorNetworkConfig {
    /// Enable Tor for all connections
    fn enable_tor(&mut self, proxy_addr: SocketAddr);
    
    /// Disable Tor
    fn disable_tor(&mut self);
}

impl TorNetworkConfig for NetworkConfig {
    fn enable_tor(&mut self, proxy_addr: SocketAddr) {
        self.use_tor = true;
        self.tor_proxy = Some(proxy_addr.to_string());
    }

    fn disable_tor(&mut self) {
        self.use_tor = false;
        self.tor_proxy = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_tor_config() {
        let proxy_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9050);
        
        let mut config = NetworkConfig {
            use_tor: false,
            tor_proxy: None,
            listen_addresses: vec![],
            bootstrap_nodes: vec![],
            use_dandelion: true,
        };

        // Enable Tor
        config.enable_tor(proxy_addr);
        assert!(config.use_tor);
        assert_eq!(config.tor_proxy, Some(proxy_addr.to_string()));

        // Disable Tor
        config.disable_tor();
        assert!(!config.use_tor);
        assert_eq!(config.tor_proxy, None);
    }

    #[tokio::test]
    async fn test_tor_handler() {
        let proxy_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9050);
        
        // This test requires a running Tor daemon
        if let Ok(handler) = TorHandler::new(proxy_addr).await {
            assert_eq!(handler.proxy_addr(), proxy_addr);
        }
    }
}