# Remote Text server

## Prerequisites

- Rust/Cargo
- pdfLaTeX (we're using the full distribution, but it's up to you — smaller distributions are more likely to cause errors due to missing packages)
- pandoc

## Running

```
cargo run
```

The server runs on port 3030, and should be accessible from anywhere

## Viewing logs

```
RUST_LOG=remote_text_server cargo run
```

## Cross-compiling for blinky/pinky/inky/clyde

```
cargo build --release --target x86_64-unknown-linux-musl
scp target/x86_64-unknown-linux-musl/release/remote-text-server inky:
```

## Clients

- The [web client](https://github.com/Remote-Text/remote-text-client)
- The [iOS app](https://github.com/Remote-Text/remote-text-ios-client)
