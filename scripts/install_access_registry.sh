echo "Installing AccessRegistry Contract..."
WASM_HASH=$(stellar contract install \
  --wasm target/wasm32v1-none/release/access_registry.wasm \
  --source factory \
  --network testnet)
echo "Registry WASM Hash: $WASM_HASH"
echo