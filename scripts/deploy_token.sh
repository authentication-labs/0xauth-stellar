#!/bin/bash

# Deploy token
echo "Deploying token contract..."
DEPLOY_OUTPUT=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/token.wasm \
  --source factory \
  --network testnet)
echo "Deploy Output:"
echo "$DEPLOY_OUTPUT"
echo

# Output Contract ID
CONTRACT_ID=$DEPLOY_OUTPUT
echo "Contract ID: $CONTRACT_ID"
echo

# Initialize Claim Issuer Contract
echo "Initializing Contract..."
INITIALIZE_OUTPUT=$(stellar contract invoke \
  --id $CONTRACT_ID \
  --source factory \
  --network testnet \
  -- initialize \
  --admin  GAGYMEUBOIWGFVSBXJSGV62HCSPNEJO6FJMLQDVPKN326BO6RBINRUFK \
  --decimal 7 \
  --name "XBENJI" \
  --symbol "XBENJI"
 )
echo "Initialize Output:"
echo "$INITIALIZE_OUTPUT"
echo

echo "Script completed."
