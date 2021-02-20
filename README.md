# Language Server Proxy

This program will act as an easy-to-use, remote, middleman between any Language Server and your own Language Server compliant client.

Your code and language server are installed on your machine, enabling any mobile app or consumer of the proxy to immediately support your machine's toolchain.

Language Server notifications sent to the proxy (`textDocument/didChange` and `workspace/didCreateFiles`) are intercepted by the proxy to provide file synchronization between client edits to the proxy's files.

Endpoints in the proxy can be used to run and stop the running of code, which is not provided by the Language Server Protocol (`code/run/{source path}` and `code/kill`)

Made primarily with Rust, Actix Web, and Tokio.

## Features

- Remote file synchronization
- Websocket connection to Language Server Protocol compliant server
- Remote code compilation and execution
- Remote input to running code via proxy endpoint

## Missing Features

- Security (this has mainly been targeted at private usage)

## How to use

### Running

#### Program Dependencies

- a language server
  - Currently only the [Eclipse Java Language Server](https://github.com/eclipse/eclipse.jdt.ls) is officially supported
    - At *least* Java 11 to run the server
    - You could build it yourself, or download the *latest* (at least 0.60, but old may work) binary from their [releases](https://download.eclipse.org/jdtls/snapshots/?d) (**recommended**)
- a Language Server Proxy binary (you can find latest releases on this repo)

#### Program Instructions

```default
USAGE:
    lsp_proxy.exe [OPTIONS] --codebase-path --lang-server-path  --language

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --codebase-path
    -s, --lang-server-path

    -l, --language <language>
    -p, --port <port> [default: 8001]
```

- `codebase-path` is the path to the directory you want the language server to run on
- `lang-server-path` is the path to the language server binary (currently only Java supported)
- `language` is a value that tells the proxy what language is being used (refer to [this table](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocumentItem))
- `port` is the port that the proxy should listen for requests on

### Building

#### Build Dependencies

- [Rust](https://www.rust-lang.org/learn/get-started) (this will also install Cargo, the Rust build tool)

#### Build Instructions

- `cargo run` will build and run the proxy in debug mode
- `cargo run --` will build and run the proxy in debug mode, with program arguments added in after the `--` (e.g. `cargo run -- -h`)
- `cargo test` will run the test suite
- `cargo build --release` will build a binary in release mode that you can find in `/target`
