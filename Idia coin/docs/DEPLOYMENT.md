# Idia Core Deployment Guide

This document provides detailed instructions for deploying and operating an Idia node securely.

## System Requirements

### Minimum Requirements
- CPU: 2 cores
- RAM: 4GB
- Storage: 100GB SSD
- Network: 5 Mbps upload/download

### Recommended Requirements
- CPU: 4+ cores
- RAM: 8GB+
- Storage: 500GB SSD
- Network: 10+ Mbps upload/download

## Security Setup

### Operating System
1. Use a clean installation of Linux (Ubuntu 20.04 LTS recommended)
2. Keep system updated:
   ```bash
   apt update && apt upgrade -y
   ```
3. Enable firewall:
   ```bash
   ufw enable
   ufw allow 8080/tcp  # P2P port
   ```

### User Setup
1. Create dedicated user:
   ```bash
   adduser idia
   usermod -aG sudo idia
   ```
2. Configure SSH:
   ```bash
   # /etc/ssh/sshd_config
   PermitRootLogin no
   PasswordAuthentication no
   ```

## Installation

### Dependencies
```bash
# Install build dependencies
apt install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Tor (optional)
apt install -y tor
```

### Build
```bash
git clone https://github.com/your-org/idia-core.git
cd idia-core
cargo build --release
```

## Configuration

### Node Configuration
```toml
# /etc/idia/config.toml

[network]
# Network configuration
listen_addresses = ["0.0.0.0:8080"]
external_address = "your-public-ip:8080"
bootstrap_nodes = [
    "/ip4/1.2.3.4/tcp/8080/p2p/QmBootstrapNode1",
    "/ip4/5.6.7.8/tcp/8080/p2p/QmBootstrapNode2"
]
use_dandelion = true

[privacy]
# Privacy features
use_tor = true
tor_proxy = "127.0.0.1:9050"
ring_size = 11

[storage]
# Data storage
data_dir = "/var/lib/idia"
db_cache_size = 1024  # MB

[rpc]
# RPC interface
enabled = true
bind_address = "127.0.0.1:8081"
username = "rpc_user"
password = "strong_password"
```

### Tor Configuration
```
# /etc/tor/torrc

SOCKSPort 9050
ControlPort 9051
CookieAuthentication 1
```

## Running the Node

### Systemd Service
```ini
# /etc/systemd/system/idia.service

[Unit]
Description=Idia Core Node
After=network.target

[Service]
Type=simple
User=idia
Group=idia
ExecStart=/usr/local/bin/idia-core --config /etc/idia/config.toml
Restart=always
RestartSec=30
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### Start Service
```bash
systemctl enable idia
systemctl start idia
```

## Monitoring

### Log Monitoring
```bash
journalctl -u idia -f
```

### Metrics
Access built-in metrics (respects privacy):
```bash
curl http://localhost:8081/metrics
```

### Health Checks
```bash
# Check node status
idia-cli status

# Check peer count
idia-cli peers

# Check sync status
idia-cli sync
```

## Backup Procedures

### Wallet Backup
1. Stop the node
2. Backup wallet directory:
   ```bash
   tar czf wallet-backup.tar.gz /var/lib/idia/wallet
   ```
3. Store backup securely offline

### Database Backup
```bash
# Create snapshot
idia-cli db snapshot

# Backup snapshot
tar czf db-backup.tar.gz /var/lib/idia/snapshots
```

## Security Maintenance

### Regular Updates
```bash
# Update system
apt update && apt upgrade -y

# Update Idia
cd /path/to/idia
git pull
cargo build --release
systemctl restart idia
```

### Key Management
- Rotate RPC credentials regularly
- Use hardware security modules for critical keys
- Implement key backup procedures

### Network Security
- Monitor for unusual traffic patterns
- Keep firewall rules updated
- Regular security audits

## Troubleshooting

### Common Issues

1. Node won't start
   ```bash
   # Check logs
   journalctl -u idia -n 100

   # Check permissions
   ls -la /var/lib/idia
   ```

2. Sync issues
   ```bash
   # Reset peer database
   idia-cli peers reset

   # Check network connectivity
   idia-cli network test
   ```

3. Performance issues
   ```bash
   # Check resource usage
   top -u idia
   df -h /var/lib/idia
   ```

### Support Resources
- GitHub Issues
- Developer Documentation
- Community Forums

## Compliance

### Privacy Considerations
- Do not log user IPs
- No transaction correlation
- Respect user privacy settings

### Legal Requirements
- Review local regulations
- Implement required controls
- Document compliance measures

## Recovery Procedures

### Node Recovery
1. Stop service
2. Backup data
3. Reset state
4. Restore from backup
5. Restart service

### Network Recovery
1. Bootstrap from trusted nodes
2. Verify blockchain integrity
3. Re-establish peer connections

## Performance Tuning

### System Tuning
```bash
# File system tuning
echo "vm.swappiness=10" >> /etc/sysctl.conf
echo "vm.dirty_ratio=30" >> /etc/sysctl.conf

# Network tuning
echo "net.core.rmem_max=16777216" >> /etc/sysctl.conf
echo "net.core.wmem_max=16777216" >> /etc/sysctl.conf
```

### Database Optimization
```toml
[database]
cache_size = "2G"
max_open_files = 10000
compression_type = "lz4"
```

## Appendix

### Command Reference
```bash
idia-cli help                # Show help
idia-cli version            # Show version
idia-cli status             # Node status
idia-cli peers list         # List peers
idia-cli wallet balance     # Show balance
idia-cli tx send           # Send transaction
```

### File Locations
- Config: `/etc/idia/config.toml`
- Data: `/var/lib/idia/`
- Logs: `journalctl -u idia`
- Wallet: `/var/lib/idia/wallet/`