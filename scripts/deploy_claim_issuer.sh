#!/bin/bash

# Deploy Claim Issuer Contract with the wasm hash
echo "Deploying Claim Issuer Contract..."
DEPLOY_OUTPUT=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/claim_issuer.wasm \
  --source issuer \
  --network testnet)
echo "Deploy Output:"
echo "$DEPLOY_OUTPUT"
echo

# Output Claim Issuer Contract ID
CLAIM_ISSUER_CONTRACT_ID=$DEPLOY_OUTPUT
echo "Claim Issuer Contract ID: $CLAIM_ISSUER_CONTRACT_ID"
echo

# Initialize Claim Issuer Contract
echo "Initializing Claim Issuer Contract..."
INITIALIZE_OUTPUT=$(soroban contract invoke \
  --id $CLAIM_ISSUER_CONTRACT_ID \
  --source issuer \
  --network testnet \
  -- \
  initialize --initial_management_key GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C)
echo "Initialize Output:"
echo "$INITIALIZE_OUTPUT"
echo

# Add Claim Issuer Claim Key
echo "Adding Claim Issuer Claim Key..."
ADD_KEY_OUTPUT=$(soroban contract invoke \
  --id $CLAIM_ISSUER_CONTRACT_ID \
  --source issuer \
  --network testnet \
  -- \
  add_key --manager GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C --key GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C --purpose 3 --key_type 1)
echo "Add Key Output:"
echo "$ADD_KEY_OUTPUT"
echo

echo "Script completed."
