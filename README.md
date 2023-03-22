# Remote Text server

## Prerequisites

Rust and Cargo

## Running

```
cargo run
```

The server runs on port 3030, and should be accessible from anywhere

## Viewing logs

```
RUST_LOG=remote-text-server::api cargo run
```

## Cross-compiling for blinky

```
cargo build --release --target x86_64-unknown-linux-musl
scp target/x86_64-unknown-linux-musl/release/remote-text-server blinky:
```
