#!/bin/sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./web/ --target web ./target/wasm32-unknown-unknown/release/jamgame.wasm

