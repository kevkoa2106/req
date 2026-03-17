# req

A lightweight command-line HTTP client written in Rust. Define your HTTP requests in `.rest` files and execute them from the terminal with pretty-printed JSON responses.

## Features

- Parse and execute HTTP requests from `.rest` or `.http` files
- Support for **GET**, **POST**,**PUT**,**PATCH**, **DELETE** methods
- Custom headers (e.g. `Content-Type`)
- JSON request bodies
- Pretty-printed JSON responses
- Async execution with Tokio

## Usage

1. Define your request in a `http.rest` file:

```http
POST https://api.restful-api.dev/objects
Content-Type: application/json

{
  "name": "Apple MacBook Pro 16",
  "data": {
    "year": 2019,
    "price": 1849.99,
    "CPU model": "Intel Core i9",
    "Hard disk size": "1 TB"
  }
}
```

2. Run:

```sh
cargo run
```

The response will be printed as formatted JSON to stdout.

## .rest File Format

```
METHOD URL
Header-Name: Header-Value (optional)

Body (optional, separated by a blank line)
```

## Building

```sh
cargo build --release
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client |
| `tokio` | Async runtime |
| `formatjson` | JSON pretty-printing |
| `ratatui` | Terminal UI (planned) |
| `crossterm` | Terminal handling (planned) |

## Roadmap

- [x] Support all HTTP methods (PUT, DELETE, PATCH, GET, POST)
- [ ] Multiple requests per file (separated by `###`)
- [ ] HTTPS support in parser
- [ ] Multiple headers per request
- [ ] Variable substitution (`{{variable}}`)
- [ ] CLI arguments (file path, verbose mode)
- [ ] Response metadata display (status code, headers)
- [ ] Interactive TUI mode

## License

MIT
