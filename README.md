```sh
cargo build --release --target wasm32-unknown-unknown --package clearing_house
candid-extractor target/wasm32-unknown-unknown/release/clearing_house.wasm > clearing_house.did
```