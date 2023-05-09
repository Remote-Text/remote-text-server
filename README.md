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

Creates a file on the server

Response Codes:
- 200 OK
- 400 Bad Request
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

### api/getFile (POST)

Returns a files if it exists.

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“id”: UUID,
	“hash”: String
}
```

Response:
```
{
	“id”: UUID,
	“name”: String,
	“content”: String
}
```

### api/saveFile (POST)

Saves a file

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“id”: UUID,
	“name”: String,
	“content”: String,
	“parent”: String,
	“branch”: String
}

```

Response:
```
{
	“hash”: String,
	“parent”: String
}
```

### api/deleteFile (POST)

Deletes a file if it exists

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“id”: UUID
}
```

Response: none

## Preview

### api/previewFile (POST)

Previews a file

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 418 File with no extension
- 500 Internal Server Error


Body:
```
{
	“id”: UUID,
	“hash”: String
}
```

Response:
```
{
	“log”: String,
	“state”: String*
}
```

### api/getPreview (POST)

Gets the preview of the file

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“id”: UUID,
	“hash”: String
}
```

Response: The raw file data. Can be binary, PDF, HTML, etc.

## History

### api/getHistory (POST)

Gets the history of a file

Response Codes:
- 200 OK
- 400 Bad Request
- 404 Not Found
- 413 Payload Too Large
- 500 Internal Server Error


Body:
```
{
	“id”: UUID
}
```

Response:
```
{
	“commits”: [
		{
			“hash”: String,
			“parent”: String //Optional
		}
	],
	“refs”: [
		{
			“name”: String,
			“hash”: String
		}
	]
}
```
