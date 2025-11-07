#!/bin/bash

# Generate test accounts
echo "Generating test accounts..."
ALICE_KEY=$(./idia-cli account generate)
BOB_KEY=$(./idia-cli account generate)
CHARLIE_KEY=$(./idia-cli account generate)

# Fund test accounts
echo "Funding test accounts with initial balance..."
./idia-cli --testnet faucet send $ALICE_KEY 1000000
./idia-cli --testnet faucet send $BOB_KEY 1000000
./idia-cli --testnet faucet send $CHARLIE_KEY 1000000

# Set up liquidity pools
echo "Setting up test liquidity pools..."
./idia-cli --testnet dex create-pool IDIA/ETH
./idia-cli --testnet dex create-pool IDIA/USDT

# Add initial liquidity
echo "Adding initial liquidity..."
./idia-cli --testnet dex add-liquidity \
    --pool IDIA/ETH \
    --amount-a 100000 \
    --amount-b 50 \
    --account $ALICE_KEY

./idia-cli --testnet dex add-liquidity \
    --pool IDIA/USDT \
    --amount-a 100000 \
    --amount-b 100000 \
    --account $BOB_KEY

# Create some test transactions
echo "Creating test transactions..."
./idia-cli --testnet tx send \
    --from $ALICE_KEY \
    --to $BOB_KEY \
    --amount 1000 \
    --privacy-level high

./idia-cli --testnet tx send \
    --from $BOB_KEY \
    --to $CHARLIE_KEY \
    --amount 500 \
    --privacy-level medium

# Set up staking
echo "Setting up staking..."
./idia-cli --testnet stake \
    --amount 50000 \
    --lock-period 30 \
    --account $CHARLIE_KEY

# Create test cross-chain bridge transaction
echo "Creating test bridge transaction..."
./idia-cli --testnet bridge \
    --to ethereum \
    --amount 1000 \
    --account $ALICE_KEY

echo "Preview environment is ready!"
echo "Access points:"
echo "- Explorer: http://localhost:3000"
echo "- Wallet UI: http://localhost:3001"
echo "- DEX UI: http://localhost:3002"
echo "- Metrics: http://localhost:3003"
echo ""
echo "Test accounts:"
echo "Alice: $ALICE_KEY"
echo "Bob: $BOB_KEY"
echo "Charlie: $CHARLIE_KEY"