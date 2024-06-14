# ReDeFi Relay Runtime

## Project Description

Custom relay runtime (based on Polkadot) with EVM compatibility support based on the Unqiue Network Frontier [fork](https://github.com/UniqueNetwork/unique-frontier).

## Build

Build the runtime by cloning this repository and running the following commands from the root directory of the repo:

```bash
 cargo build --profile=production
```

For fast runtime the feature can be used:

```bash
cargo build --profile=production --features=fast-runtime  
```

## Run

See instructions in [this repository](https://github.com/ReDeFi-Blockchain/redefi-infra).
