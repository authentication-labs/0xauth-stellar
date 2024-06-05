# 0xAuth Stellar Contracts

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── contracts
│   └── identity
│       ├── src
│       │   ├── lib.rs
│       │   ├── state.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

# Factory Deployment Process

1. First Install Indentity and get the contract wasm hash
```
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/identity.wasm \
  --source factory \
  --network testnet
```
**Output WASM Hash**
```
55c768523facf254d87cc941c564be848b2f1c84366ba0f91373e93ea0681706
```

2. Deploy Factory Contract
```
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/factory.wasm \
  --source factory \
  --network testnet
```

**Output Factory Contract ID**
```
CCZXKVMRGJBWXHGIDCNGBJHFNNZFE3E2D55MSDWNUP3BM2L44PZFLMOR
```

3. Initialize Factory Contract
Get the factory wallet address: `GAGYMEUBOIWGFVSBXJSGV62HCSPNEJO6FJMLQDVPKN326BO6RBINRUFK`
```
soroban contract invoke \
  --id CCZXKVMRGJBWXHGIDCNGBJHFNNZFE3E2D55MSDWNUP3BM2L44PZFLMOR \
  --source factory \
  --network testnet \
  -- \
  initialize --owner GAGYMEUBOIWGFVSBXJSGV62HCSPNEJO6FJMLQDVPKN326BO6RBINRUFK
```

4. Deploy Identity Contract Through Factory
It expects the following parameters:
- `wasm_hash: BytesN<32>`: The hash of the identity contract
- `wallet: Address`: The address of the wallet that will own the identity contract
- `salt: BytesN<32>`: A random number to ensure the identity contract address is unique
- `init_fn: Symbol`: The function to call on the identity contract after deployment
- `init_args: Vec<Val>`: The arguments to pass to the init_fn

Invoke Command:
```
soroban contract invoke \
  --id CCZXKVMRGJBWXHGIDCNGBJHFNNZFE3E2D55MSDWNUP3BM2L44PZFLMOR \
  --source factory \
  --network testnet \
  -- \
  create_identity --wasm_hash 55c768523facf254d87cc941c564be848b2f1c84366ba0f91373e93ea0681706 --wallet GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB --salt 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abdcfe --init_fn initialize --init_args '[{"address": "GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB"}]'
```

**Output Identity Contract ID**
```
["CD2JGMD27J4AYYE44HB7PYPZTUQ2RLVP5TJDXEKDVJIAP5SMWK33FQBS",null]
```

5. Get Identity Contract State
Now we can call `get_initialized` on the identity contract to get the state of the contract
```
soroban contract invoke \
  --id CD2JGMD27J4AYYE44HB7PYPZTUQ2RLVP5TJDXEKDVJIAP5SMWK33FQBS \
  --source factory \
  --network testnet \
  -- \
  get_initialized
```

Output: `true`