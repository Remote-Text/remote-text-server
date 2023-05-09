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

## Cross-testing for *inky

```
 cargo test --target x86_64-unknown-linux-musl --no-run
 scp target/x86_64-unknown-linux-musl/debug/deps/remote_text_server-00dbdac0b2c64b5a inky: #determined by output of `cargo test`; don't copy and paste this line
 ```

## Clients

- The [web client](https://github.com/Remote-Text/remote-text-client)
- The [iOS app](https://github.com/Remote-Text/remote-text-ios-client)

# Api Endpoints

## Files

### /listFiles (GET)

Returns a list of the files on the server.

Responses
- 200 OK
- 500 Internal Server Error

Example Response
```
Response:
HTTP/1.1 200 OK
content-type: application/json
content-length: 141

[
 {
 "name":"README.md",
 "id":"ed8b1618-97b1-4a69-a080-4207152640ea",
 "edited_time":"2023-05-04T23:30:12Z",
 "created_time":"2023-05-04T23:30:12Z"
 }
]
```




