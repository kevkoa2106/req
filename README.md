# req

[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![GitHub stars](https://img.shields.io/github/stars/kevkoa2106/req)](https://github.com/kevkoa2106/req/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/kevkoa2106/req)](https://github.com/kevkoa2106/req/issues)

A lightweight command-line HTTP client written in Rust. Define your HTTP requests in `.rest` and `.http` files and execute them from the terminal with pretty-printed JSON responses.

![Demo](assets/demo.gif)

## Features

- Parse and execute HTTP requests from `.rest` or `.http` files
- **Multiple requests per file** separated by `###`
- Support for **GET**, **POST**, **PUT**, **PATCH**, **DELETE** methods
- Custom headers (e.g. `Content-Type`)
- JSON request bodies
- Pretty-printed JSON responses
- Async execution with Tokio
- Environment variables via `http-client.env.json` with `{{variable}}` substitution
- Private environment overrides via `http-client.private.env.json`
- Interactive TUI mode with tabbed view for multiple requests

## Installation

### From Homebrew 🍺

```sh
brew install kevkoa2106/tap/req
```

### From GitHub Releases

Download the latest prebuilt binary for your platform from the [Releases page](https://github.com/kevkoa2106/req/releases):

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) 🍎 | [req-aarch64-apple-darwin.tar.gz](https://github.com/kevkoa2106/req/releases/download/v0.1.3/req-aarch64-apple-darwin.tar.gz) |
| macOS (Intel) 🍎 | [req-x86_64-apple-darwin.tar.gz](https://github.com/kevkoa2106/req/releases/download/v0.1.3/req-x86_64-apple-darwin.tar.gz) |
| Linux (x86_64) 🐧 | [req-x86_64-unknown-linux-gnu.tar.gz](https://github.com/kevkoa2106/req/releases/download/v0.1.3/req-x86_64-unknown-linux-gnu.tar.gz) |
| Linux (ARM) 🐧 | [req-aarch64-unknown-linux-gnu.tar.gz](https://github.com/kevkoa2106/req/releases/download/v0.1.3/req-aarch64-unknown-linux-gnu.tar.gz) |
| Windows (x86_64) | [req.exe](https://github.com/kevkoa2106/req/releases/download/v0.1.3/req.exe) |

### From source

```sh
git clone https://github.com/kevkoa2106/req.git
cd req
cargo build --release
```

## Usage

1. Define your request(s) in a `http.rest` file:

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

###

GET https://api.restful-api.dev/objects
```

Use `###` to separate multiple requests in a single file. Each request will be executed in order.

2. Run:

```sh
req http.rest
```

The response(s) will be printed as formatted JSON to stdout, separated by `###`.

## Arguments

| Argument | Short | Long | Description | Default |
|----------|-------|------|-------------|---------|
| `<INPUT>` | | | The input `.rest` or `.http` file | *required* |
| `-v` | `-v` | `--verbose` | Shows full error output | `false` |
| `-t` | `-t` | `--tui` | Launch interactive TUI mode | `false` |
| `-e` | `-e` | `--env` | Environment name from `http-client.env.json` | `development` |
| | | `--private` | Load `http-client.private.env.json` | `false` |

**Examples:**

```sh
req api.rest                          # basic request
req api.rest -v                       # with verbose errors
req api.rest -t                       # interactive TUI mode
req api.rest --env production         # use production environment
req api.rest --env staging --private  # staging env with private variables
```

## Environment Variables

Create an `http-client.env.json` file alongside your `.rest` file to define variables per environment:

```json
{
  "development": {
    "host": "localhost",
    "id-value": 12345
  },
  "production": {
    "host": "example.com",
    "id-value": 6789
  }
}
```

Use `{{variable}}` placeholders in your `.rest` file:

```http
GET http://{{host}}/api/json/get?id={{id-value}}
Content-Type: application/json
```

Run with a specific environment using `--env`:

```sh
req http.rest --env production
```

The default environment is `development`.

### Private Variables

For sensitive values like passwords and API keys, create an `http-client.private.env.json` file:

```json
{
  "development": {
    "username": "dev-user",
    "password": "dev-secret"
  }
}
```

Use the `--private` flag to load private variables (they override values from `http-client.env.json`):

```sh
req http.rest --env production --private
```

## .rest File Format

```
METHOD URL
Header-Name: Header-Value (optional)

Body (optional, separated by a blank line)

###

METHOD URL
...
```

Use `###` on its own line to separate multiple requests. Each request can have its own method, URL, headers, and body. In TUI mode, each request gets its own tab — press `1`-`9` to switch between them.

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
| `ratatui` | Terminal UI |
| `crossterm` | Terminal handling |
| `serde_json` | JSON parsing for env files |
| `regex` | Variable substitution |

## Roadmap

- [x] Support all HTTP methods (PUT, DELETE, PATCH, GET, POST)
- [x] Multiple requests per file (separated by `###`)
- [x] HTTPS support in parser
- [x] Multiple headers per request
- [x] Variable substitution (`{{variable}}`)
- [x] CLI arguments (file path, verbose mode)
- [x] Response metadata display (status code, headers)
- [x] Interactive TUI mode

## License

MIT
