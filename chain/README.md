# Substrate Weather Forecast

A blockchain for weather forecast based on [Substrate](https://github.com/paritytech/substrate).

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## Run

Purge any existing development chain state:

```bash
./target/release/weather-forecast purge-chain --dev
```

Start a development chain with:

```bash
./target/release/weather-forecast --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.