## Build for Realese

```sh
cargo build --release --target wasm32-unknown-unknown --package clearing_house
 candid-extractor target/wasm32-unknown-unknown/release/clearing_house.wasm > src/clearing_house.did
```

## Deploy Clearing with paramters

```sh

export ASSET_SYMBOL=USDT
export LEDGER_ID=lpgic-cqaaa-aaaaf-qbtiq-cai
export ADMIN=7evt6-bawqy-dprj2-qpsed-6qyrz-ghslz-jk3gz-m6u6i-5wbe4-4rixr-tqe
export INIT_CYCLES=3000000000000
dfx deploy clearing_house    --argument "(record {admin = principal \"${ADMIN}\" ; house_asset_ledger = record {ledger_id = principal \"${LEDGER_ID}\";
ledger_type = variant {ICRC} ;asset_decimals = 6};house_asset_pricing_details = record {class = variant {Cryptocurrency};symbol = \"${ASSET_SYMBOL}\"};execution_fee = 0 })" 
```
