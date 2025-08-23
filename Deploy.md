```sh
cargo build --release --target wasm32-unknown-unknown --package clearing_house
 candid-extractor target/wasm32-unknown-unknown/release/clearing_house.wasm > src/clearing_house.did
```

```sh
dfx start --clean

```

```sh
dfx stop
```

```sh
export XRC_ID=5se5w-zaaaa-aaaaf-qanmq-cai
export ASSET_SYMBOL=USD
dfx deploy clearing_house --argument "(principal \"${XRC_ID}\",record { house_asset_ledger = record {ledger_id = principal \"${XRC_ID}\";
ledger_type = variant {ICRC} ;asset_decimals = 6}; markets_tokens_ledger = record {ledger_id = principal \"${XRC_ID}\";
ledger_type = variant {RASSET} ;asset_decimals = 6};house_asset_pricing_details = record {class = variant {Cryptocurrency};symbol = \"${ASSET_SYMBOL}\"};execution_fee = 0})"
```