#!/bin/bash

# This script deploys the smart contracts to the Hedera testnet.
#
# Prerequisites:
# 1. Foundry must be installed.
# 2. You must have a .env file in the root of the project with the following variable:
#    PRIVATE_KEY=YOUR_HEDERA_TESTNET_PRIVATE_KEY

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '#' | awk '/=/ {print $1}')
fi

# Hedera Testnet RPC URL
HEDERA_TESTNET_RPC_URL="https://testnet.hashio.io/api"

# Deploy the contracts
forge script contracts/script/Deploy.s.sol:Deploy --rpc-url $HEDERA_TESTNET_RPC_URL --broadcast
