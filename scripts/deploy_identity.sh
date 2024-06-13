#!/bin/bash

generate_random_salt() {
  openssl rand -hex 32
}

# Step 1: Install Identity Contract and get the wasm hash
echo "Installing Identity Contract..."
IDENTITY_WASM_HASH=$(soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/identity.wasm \
  --source factory \
  --network testnet)
echo "Identity WASM Hash: $IDENTITY_WASM_HASH"
echo

# Step 2: Deploy Factory Contract
echo "Deploying Factory Contract..."
FACTORY_CONTRACT_ID=$(soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/factory.wasm \
  --source factory \
  --network testnet)
echo "Factory Contract ID: $FACTORY_CONTRACT_ID"
echo


# FACTORY_CONTRACT_ID="CDRHGLONE56BNO7DOTYYG64V2D75XWCJTVWGQRDS5GBFV5UCPR2MSOTX"
# Step 3: Initialize Factory Contract
FACTORY_WALLET_ADDRESS="GAGYMEUBOIWGFVSBXJSGV62HCSPNEJO6FJMLQDVPKN326BO6RBINRUFK"
echo "Initializing Factory Contract..."
INITIALIZE_FACTORY_OUTPUT=$(soroban contract invoke \
  --id $FACTORY_CONTRACT_ID \
  --source factory \
  --network testnet \
  -- \
  initialize --owner $FACTORY_WALLET_ADDRESS)
echo "Initialize Factory Output:"
echo "$INITIALIZE_FACTORY_OUTPUT"
echo

# Step 4: Deploy Identity Contract Through Factory
WALLET_ADDRESS="GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB"
SALT=$(generate_random_salt)
INIT_FN="initialize"
INIT_ARGS='[{"address":"GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB"}]'
echo "Deploying Identity Contract through Factory..."
IDENTITY_CONTRACT_DEPLOY_OUTPUT=$(soroban contract invoke \
  --id $FACTORY_CONTRACT_ID \
  --source factory \
  --network testnet \
  -- \
  create_identity --wasm_hash $IDENTITY_WASM_HASH --wallet $WALLET_ADDRESS --salt $SALT --init_fn $INIT_FN --init_args "$INIT_ARGS")
IDENTITY_CONTRACT_ID=$(echo $IDENTITY_CONTRACT_DEPLOY_OUTPUT | grep -oP '(?<=\[")\w+')
echo "Identity Contract ID: $IDENTITY_CONTRACT_ID"
echo


echo "Adding Identity Contract Claim Key..."
IDENTITY_STATE_OUTPUT=$(soroban contract invoke \
  --id $IDENTITY_CONTRACT_ID \
  --source manager \
  --network testnet \
  -- \
  add_key --manager GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB --key GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB --purpose 3 --key_type 1)
echo "Add Key Output:"
echo "$ADD_KEY_OUTPUT"
echo

echo "Script completed."
