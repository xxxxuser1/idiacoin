# Privacy Coin Design Document

## Overview

This document outlines the design and implementation plan for a privacy-focused cryptocurrency that combines multiple privacy primitives to achieve strong transaction privacy. The design incorporates stealth addresses, ring signatures, confidential transactions, and optional zero-knowledge proofs to provide comprehensive privacy protection.

## Contract

### Inputs
- Requirements for ring signatures, stealth addresses, zero-knowledge proofs, confidential transactions, and network privacy

### Outputs
- Complete design and implementation plan
- Recommended libraries and technology stack
- Prototype implementation steps
- Testing and audit checklist

### Error Modes
- Cryptographic misuse
- Linkability leaks
- Network deanonymization
- Regulatory/legal blocking

### Success Criteria
- Sender/receiver unlinkability
- Hidden transaction amounts
- Practical performance (transaction size/verification time)
- Clear testing and audit plan

## High-Level Architecture

### Consensus Layer
- Choice between Proof of Work (PoW) or Proof of Stake (PoS)
- Maintains block creation rules
- Privacy primitives operate within transactions

### P2P Layer
- Implementation using libp2p
- Dandelion++ protocol for transaction propagation
- Optional Tor/I2P integration via SOCKS5 proxy

### Transaction Layer
- UTXO-based model for natural isolation of outputs
- Each UTXO contains commitments and zero-knowledge proofs instead of explicit amounts/addresses

### Crypto Primitives
- Key derivation and stealth addresses
- Ring signatures for input anonymity
- Pedersen commitments + Bulletproofs for confidential amounts
- Optional zk-SNARKs/PLONK for complex rules or shielded pools

### Wallets
- Local key storage
- Deterministic seed (BIP39 or custom)
- Per-transaction one-time addresses
- Decoy selection logic
- Coin selection algorithms

### Explorer
- Optional node-side wallet-view only
- No global address index by default
- User-controlled encrypted view-keys

## Threat Model

### Adversary Capabilities
- Network observer (global passive)
- Active attacker controlling nodes
- Blockchain analyst
- Law enforcement (with subpoena)
- Correlation via IP addresses
- Compromised wallets

### Security Goals
- Prevent linking sender to receiver
- Hide transaction amounts
- Resist chain analysis
- Prevent network-level deanonymization

### Assumptions
- Correct usage of cryptographic primitives
- Honest-majority assumptions for consensus
- User practices influence privacy effectiveness

## Privacy Primitives

### Ring Signatures (Monero-style)

#### Description
Mix real input with decoys to hide which output is spent. Uses CLSAG (Concise Linkable Spontaneous Anonymous Group) signatures for smaller size and faster verification.

#### Implementation Notes
- Use curve25519/ed25519-friendly libraries
- Implement good decoy selection algorithm to avoid temporal linkability
- Use one-time ring signatures with key images to prevent double-spending

#### Trade-offs
- Smaller anonymity sets provide weaker privacy
- Larger rings increase transaction size and verification time

### Stealth Addresses (One-time Addresses)

#### Description
Recipient publishes a public view key and spend key. Sender derives a unique one-time public key per transaction using ECDH.

#### Implementation Notes
- Use ECDH on the same curve used elsewhere
- Store metadata so recipient can scan and recover funds
- Implement view-only wallet functionality

#### Trade-offs
- Scanning cost for wallets
- Potential leakage if view keys are shared

### Confidential Transactions (Pedersen Commitments + Range Proofs)

#### Description
Hide amounts with Pedersen commitments C = aG + rH. Use Bulletproofs range proofs to ensure sums validate balance without revealing amounts.

#### Implementation Notes
- Use curve25519-friendly Bulletproofs implementations
- Maintain invariants: sum(inputs) = sum(outputs) + fee (in commitments domain)

#### Trade-offs
- Increased transaction size and verification cost
- Bulletproofs significantly reduce range-proof size compared to older approaches

### Zero-Knowledge Proofs / zk-SNARKs (Zcash-style)

#### Description
Prove transaction validity (balances, no double-spend) without revealing amounts/addresses. Can create shielded pools for even stronger privacy.

#### Choices
- Groth16 (small proofs, requires trusted setup)
- PLONK/Marlin/Plonk-like (universal setup, slightly larger proofs)

#### Implementation Notes
- Choose appropriate library based on requirements
- Account for circuit development complexity and proving time
- Consider trusted setup requirements

#### Trade-offs
- Stronger privacy but increased complexity
- Larger build size and longer proving times
- Potential regulatory scrutiny

## Network Privacy

### Tor / I2P Integration

#### Outbound Connections
- Use SOCKS5 proxy by default for node's outbound connections
- Optional enforcement for privacy-conscious nodes

#### Inbound Connections
- Run as a Tor hidden service to accept incoming connections without revealing IP

#### Implementation Notes
- Provide config toggle to bind to SOCKS5
- Advertise onion address in node configuration
- Prevent DNS leaks
- Bind RPC/Wallet interfaces to localhost only

#### Trade-offs
- Higher latency
- Potential denial-of-service via Tor
- Challenges for bootstrap and peer discovery

### Dandelion++ (Transaction Propagation)

#### Description
Two-phase forwarding to hide transaction origin before public broadcast:
1. Stem phase: Pick a short random path where transactions are relayed privately
2. Fluff phase: Broadcast transaction publicly after random time

#### Implementation Notes
- Implement failover and timeout mechanisms to prevent black-holing
- Ensure proper transition between stem and fluff phases

#### Trade-offs
- Small delays in transaction propagation
- Implementation complexity
- Prevents simple origin inference from gossip timings

## Consensus: PoW vs PoS

### Proof of Work
#### Pros
- Well-understood and battle-tested
- Permissionless participation
- Resistant to certain types of attacks

#### Cons
- Energy consumption costs
- Potential centralization via mining pools
- Slower block times

### Proof of Stake
#### Pros
- Energy efficient
- Potentially faster finality
- Lower hardware requirements

#### Cons
- Complex economic mechanisms
- Stake-based attacks (nothing-at-stake problem)
- Slashing complexity

### Privacy Note
Consensus choice doesn't strongly affect transaction privacy primitives but impacts overall architecture, performance, and attack surfaces.

## Data Model: UTXO vs Account

### UTXO Model (Recommended)
#### Advantages for Privacy Coins
- Natural isolation of outputs
- Easier to implement one-time addresses
- Better alignment with commitment schemes

#### Disadvantages
- More complex to implement account-like features
- Storage overhead for unspent outputs

### Account Model
#### Disadvantages for Privacy Coins
- Leaks balance at address
- Harder to adapt to per-transaction stealth addresses

## Technology Stack & Recommended Libraries

### Primary Language: Rust
Chosen for safety, strong crypto ecosystem, and performance.

#### Core Libraries
- `curve25519-dalek`: Elliptic curve operations
- `bulletproofs`: Range proofs implementation
- `arkworks-rs`: Zero-knowledge proof libraries
- `libp2p-rust`: P2P networking
- `serde`: Serialization framework

#### Alternative Languages
- C++: Reference implementation from Monero
- Go: Fast prototyping and network services

## Wallet UX & Operational Considerations

### Critical Requirements
- Avoid leaking linking metadata
- No address reuse
- Careful output combination to preserve anonymity

### Features
- Local indexing and scanning
- View-only wallet support
- Intelligent coin selection with decoy management
- User warnings for privacy-sensitive operations

### Mobile Considerations
- SPV-light clients must avoid revealing which outputs belong to user
- Use Bloom/compact filters carefully
- Support for private remote nodes via Tor

## Testing, Audits, and Verification

### Test Categories
1. Unit tests for crypto primitives with test vectors
2. Fuzz testing for transaction parsing
3. Property-based tests for conservation laws
4. Benchmarking: transaction size, verification time, memory usage

### Audit Requirements
- Third-party cryptographic audit before mainnet release
- Formal verification for critical modules
- Reproducible builds for transparency

## Performance & Sizing Guidance

### Target Metrics
- Bulletproof sizes per output: Few hundred bytes
- Ring sizes: 11+ for adequate privacy
- zk-SNARK proof sizes: 100-200 bytes (Groth16)
- Verification time: Must fit target TPS requirements

### Optimization Strategies
- Aggregate proofs to reduce size
- Optimize cryptographic operations
- Implement efficient serialization

## Legal & Ethical Considerations

### Regulatory Awareness
- Privacy technology can attract regulatory attention
- Consider jurisdictional restrictions before launch
- Consult legal counsel for compliance requirements

### Compliance Features
- Optional view-keys for law enforcement with user consent
- Balance privacy with legitimate compliance needs
- Clear user education on responsible usage

## Minimal Prototype Plan

### Language
Rust (primary), with potential Go components for networking

### Core Components
1. Tiny UTXO model implementation
2. Key generation (ed25519/ristretto)
3. Stealth address derivation (ECDH → one-time pubkey)
4. Pedersen commitments for amounts
5. Simple range proofs via Bulletproofs crate
6. Minimal ring signature implementation (MLSAG/CLSAG)
7. Test harness for end-to-end validation

### Implementation Steps
1. Create wallet A→B transaction flow
2. Verify commitment balance conservation
3. Validate receiver can scan and recover funds
4. Add Dandelion++ stub in P2P messaging
5. Integrate Tor SOCKS5 support

## Edge Cases & Pitfalls

### Top 8 Risks
1. Poor decoy selection leading to temporal linkability
2. Reusing addresses or combining outputs causing deanonymization
3. Incorrect commitment blinding resulting in fund loss
4. Range-proof aggregation mistakes causing verification failures
5. Trusted setup misuse (Groth16) if used carelessly
6. Network leaks from public RPC endpoints
7. Wallet backup failures leading to irrecoverable loss
8. Side-channel leaks from timing or telemetry

## Next Steps

1. Complete detailed technical specifications for each component
2. Begin prototype implementation focusing on core privacy primitives
3. Establish testing framework and benchmarks
4. Engage security auditors for review process
5. Develop community and documentation