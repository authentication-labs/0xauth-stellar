#!/bin/bash

echo "Funding Manager Keys..."
soroban keys fund manager --network testnet

echo "Funding Factory Keys..."
soroban keys fund factory --network testnet

echo "Funding Issuer Keys..."
soroban keys fund issuer --network testnet