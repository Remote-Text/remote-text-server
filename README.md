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

### api/listFiles (GET)

Returns a list of the files on the server.

Response Codes:
- 200 OK
- 500 Internal Server Error

Response
```
[
	{
		“id”: UUID,
		“name”: String,
		“edited_time”: Date,
		“created_time”: Date
	}
]
```

### api/createFile (POST)

Returns a file if one exists

Response Codes:
- 200 OK
- 400 Bad REquest
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“name”: String,
	“content”: String //Optional
}
```

Response:
```
{
		“id”: UUID,
		“name”: String,
		“hash”: String,
		“created_time”: Date
	}

```
