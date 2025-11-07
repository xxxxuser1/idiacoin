# Idia Core

A privacy-focused cryptocurrency implementation in Rust, featuring advanced cryptographic primitives and network-level privacy protections.

## Features

### Privacy Protections

- **Ring Signatures (MLSAG)**: Hide transaction sources by mixing with decoys
- **Stealth Addresses**: Generate unique one-time addresses for each transaction
- **Confidential Transactions**: Hide transaction amounts using Pedersen commitments
- **Range Proofs**: Verify transaction validity without revealing amounts using Bulletproofs
- **Network Privacy**: Tor integration and Dandelion++ protocol for network-level anonymity

### Core Components

- **UTXO Model**: Privacy-preserving transaction model
- **Wallet Infrastructure**: Secure key storage, transaction building, and output scanning
- **P2P Network**: Decentralized peer-to-peer communication
- **Privacy-preserving Explorer**: View transactions without compromising privacy

## Architecture

### Cryptographic Primitives

- **Curve**: Curve25519 (via curve25519-dalek)
- **Commitments**: Pedersen commitments for amount hiding
- **Proofs**: Bulletproofs for range verification
- **Signatures**: MLSAG ring signatures for input anonymity

### Network Layer

- **P2P Protocol**: Built on libp2p
- **Transaction Propagation**: Dandelion++ protocol
- **Anonymity Network**: Tor integration via SOCKS5 proxy

### Privacy Features

1. **Transaction Privacy**:
   - Hidden amounts (Pedersen + Bulletproofs)
   - Hidden transaction graph (Ring signatures)
   - Recipient privacy (Stealth addresses)

2. **Network Privacy**:
   - IP address protection (Tor)
   - Transaction origin obfuscation (Dandelion++)

## Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Tor (optional, for network privacy)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/idia-core.git
   cd idia-core
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

### Configuration

1. Create a configuration file:
   ```toml
   # config.toml
   [network]
   use_tor = true
   use_dandelion = true
   listen_addresses = ["127.0.0.1:8080"]

   [wallet]
   data_dir = "./wallet"
   ring_size = 11
   ```

2. Configure Tor (optional):
   - Install Tor daemon
   - Configure SOCKS5 proxy (default: 127.0.0.1:9050)

## Usage

### Running a Node

```bash
cargo run --release -- --config config.toml
```

### Creating a Wallet

```bash
cargo run --release -- wallet create
```

### Sending Transactions

```bash
cargo run --release -- wallet send <address> <amount>
```

## Security Considerations

### Threat Model

1. **Network Surveillance**:
   - Protected by: Tor, Dandelion++
   - Assumption: Not all nodes are compromised

2. **Transaction Analysis**:
   - Protected by: Ring signatures, stealth addresses
   - Assumption: Sufficient transaction volume for anonymity

3. **Amount Privacy**:
   - Protected by: Confidential transactions
   - Assumption: Cryptographic assumptions hold

### Best Practices

1. **Wallet Security**:
   - Keep private keys offline
   - Use strong passwords
   - Regular backups

2. **Transaction Privacy**:
   - Avoid merging outputs
   - Wait between transactions
   - Use sufficient ring size

3. **Network Privacy**:
   - Always use Tor
   - Run your own node
   - Avoid revealing IP addresses

## Contributing

1. Fork the repository
2. Create your feature branch
3. Run tests: `cargo test`
4. Submit a pull request

### Development Guidelines

- Follow Rust best practices
- Add unit tests for new features
- Document public APIs
- Consider privacy implications

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test crypto
cargo test network
cargo test wallet
```

## License

[MIT License](LICENSE)

## Acknowledgments

- Monero project (ring signatures, stealth addresses)
- Grin project (MimbleWimble, rangeproofs)
- libp2p team
- Tor project

## Disclaimer

This software is provided as-is. Users are responsible for compliance with local regulations regarding cryptocurrency and privacy technology.