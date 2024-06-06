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

# Claim issuer Deployment
1. First Install Claim Issuer and get the contract wasm hash
```
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/claim_issuer.wasm \
  --source issuer \
  --network testnet
```

**Output WASM Hash**
```
b5c8b01f7228736329a266416c4798773cfe9f9cfeb9e0924cd4994dae2ba04f
```

2. Deploy Claim Issuer Contract with the wasm hash
```
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/claim_issuer.wasm \
  --source issuer \
  --network testnet
```

**Output Claim Issuer Contract ID**
```
CAV3THXYNWYNFZYUF43WJSFBQKRFQEXPFTM6A3FO6LXJ5MA3XA2OBR42
```

3. Initialize Claim Issuer Contract
```
soroban contract invoke \
  --id CAV3THXYNWYNFZYUF43WJSFBQKRFQEXPFTM6A3FO6LXJ5MA3XA2OBR42 \
  --source issuer \
  --network testnet \
  -- \
  initialize --initial_management_key GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C
```

4. Add Claim Issuer Claim Key
```
soroban contract invoke \
  --id CAV3THXYNWYNFZYUF43WJSFBQKRFQEXPFTM6A3FO6LXJ5MA3XA2OBR42 \
  --source issuer \
  --network testnet \
  -- \
  add_key --manager GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C --key GBISRSPRDGQ3OVFTQUR2FSXVJFKEJ4MKN55EVZLL7GUNERVUVKOXX74C --purpose 3 --key_type 1

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
3033769358c8b90be7b9f827424a7dbf59f487703f585e7b9e6a6fde734237e2
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
CDRHGLONE56BNO7DOTYYG64V2D75XWCJTVWGQRDS5GBFV5UCPR2MSOTX
```

3. Initialize Factory Contract
Get the factory wallet address: `GAGYMEUBOIWGFVSBXJSGV62HCSPNEJO6FJMLQDVPKN326BO6RBINRUFK`
```
soroban contract invoke \
  --id CDRHGLONE56BNO7DOTYYG64V2D75XWCJTVWGQRDS5GBFV5UCPR2MSOTX \
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
  --id CDRHGLONE56BNO7DOTYYG64V2D75XWCJTVWGQRDS5GBFV5UCPR2MSOTX \
  --source factory \
  --network testnet \
  -- \
  create_identity --wasm_hash 3033769358c8b90be7b9f827424a7dbf59f487703f585e7b9e6a6fde734237e2 --wallet GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB --salt 1023484789abcdfe0123456789abcdef0123456789abcdef0123456789abcdfe --init_fn initialize --init_args '[{"address": "GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB"}]'
```

**Output Identity Contract ID**
```
["CA7ZX2QBT7VFPLAL6G3ITILXVEULKZPXXYZ6M5HALTCIMTB4WEWI2GCK",null]
```

5. Get Identity Contract State
Now we can call `get_initialized` on the identity contract to get the state of the contract
```
soroban contract invoke \
  --id CA7ZX2QBT7VFPLAL6G3ITILXVEULKZPXXYZ6M5HALTCIMTB4WEWI2GCK \
  --source factory \
  --network testnet \
  -- \
  get_initialized
```

Output: `true`