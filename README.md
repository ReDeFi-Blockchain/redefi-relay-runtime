# ReDeFi Relay Runtime

## Project Description

Custom relay runtime (based on Polkadot) with EVM compatibility support based on the Unqiue Network Frontier [fork](https://github.com/UniqueNetwork/unique-frontier).

## Rust compiler versions

This release was built and tested against the following versions of rustc.

```
Rust Nightly: rustc 1.77.0-nightly (ef71f1047 2024-01-21)
```

Other versions may work.
Note: add targets:

```bash
rustup target add wasm32-unknown-unknown 
rustup target add x86_64-unknown-linux-musl
```

## Build

Build the runtime by cloning this repository and running the following commands from the root directory of the repo:

```bash
 cargo build --profile=production
```

For fast runtime the feature can be used:

```bash
 cargo build --profile=production --features=fast-runtime  
```
