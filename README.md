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

### Deploy Contract
```
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/identity.wasm \
  --source alice \
  --network testnet
```

### Output
`CBRRZMHM4PMPFKFIRH2HJLNGIQL2RV6VLP7FOPOGDF57VGVWEPM5IYOY`


### Invoke Contract
```
soroban contract invoke \
  --id CBRRZMHM4PMPFKFIRH2HJLNGIQL2RV6VLP7FOPOGDF57VGVWEPM5IYOY \
  --source manager \
  --network testnet \
  -- \
  initialize --initial_management_key GCKDZSO5Z2XLD4LJSA67ER3YSRBHYGRZN2PTANPK25THWKB72T3S5XSB
```
