# Language Server Proxy

This program will act as an easy-to-use, remote, middleman between any Language Server and your own Language Server compliant client.

Your code and language server are installed on your machine, enabling any mobile app or consumer of the proxy to immediately support your machine's toolchain.

Language Server notifications sent to the proxy (`textDocument/didChange` and `workspace/didCreateFiles`) are intercepted by the proxy to provide file synchronization between client edits to the proxy's files.

Endpoints in the proxy can be used to run and stop the running of code, which is not provided by the Language Server Protocol (`code/run/{source path}` and `code/kill`)

Made primarily with Rust, Actix Web, and Tokio.

## How to Run and Test

- You'll need [cargo](https://crates.io/)
- Run `cargo run`
- Run `cargo test`

## Features

- Remote file synchronization
- Websocket connection to Language Server Protocol compliant server
- Remote code compilation and execution
- Remote input to running code via proxy endpoint

## Missing Features

- Configurable language server (currently hard coded to Java)
- Security (this has mainly been made for private usage)
