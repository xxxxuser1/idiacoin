//! Idia Core - A privacy-focused cryptocurrency implementation
//! 
//! This library implements the core functionality of the Idia privacy coin,
//! including cryptographic primitives, network layer, and wallet functionality.

pub mod crypto;
pub mod network;
pub mod wallet;
pub mod types;

pub use crypto::*;
pub use network::*;
pub use wallet::*;
pub use types::*;

/// Version of the Idia protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(PROTOCOL_VERSION, "0.1.0");
    }
}
