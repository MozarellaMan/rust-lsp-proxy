# Language Server Proxy

This program aims to act as an easy-to-use, remote, middleman between any Language Server and your own Language Server compliant client.

Your code and language server are installed on your machine, enabling any mobile app or consumer of the proxy to immediately support your machine's toolchain.

Language Server notifications sent to the proxy (`textDocument/didChange` and `workspace/didCreateFiles`) are intercepted by the proxy to provide file synchronization between client edits to the proxy's files.

Endpoints in the proxy can be used to run and stop the running of code, via a websocket, which is not provided by the Language Server Protocol (`code/run/{source path}`)

Made with Rust, Actix Web, and Tokio.

Have a look at the [Architecture](/docs/ARCHITECTURE.md) document for an overview of the code.

## Features

- Remote file synchronization
- Websocket connection to Language Server Protocol compliant server
  - Currently, only the Java JDT language server has a pre-configured run command in the proxy, however it is possible to set a custom command via command line argument to run any language server of your choosing
- Remote code compilation and execution
- Remote input to running code via proxy endpoint
- Proxy is thin and lightweight in resource usage

## Currently Unimplemented

- Code execution for common languages, not just java
- Remote file deletion
- Security (this has mainly been targeted at private usage)

## Example Client

<img src="https://user-images.githubusercontent.com/48062697/110400498-a0a7aa80-806f-11eb-9929-59c6b4062f2c.gif" width="280"> <img src="https://user-images.githubusercontent.com/48062697/110399225-5e7d6980-806d-11eb-8d69-27befc1f67a9.png" width="280">

The [app](https://github.com/MozarellaMan/Mobile-LSP-Client) shown above connects to the proxy; uses communication with the language server to provide language features such as code diagnostics (warning about resource leak) and uses the proxy to provide file synchronisation and the ability to run and send input to the running source code remotely. This is just an example, a different client consuming this proxy can look totally different or run on a different device.

## How to use

### Running

#### Program Dependencies

- a language server
  - Currently only the [Eclipse Java Language Server](https://github.com/eclipse/eclipse.jdt.ls) is officially supported
    - At *least* Java 11 to run the server
    - You could build it yourself, or download the *latest* (at least 0.60, but old may work) binary from their [releases](https://download.eclipse.org/jdtls/snapshots/?d) (**recommended**)
- a Language Server Proxy binary (you can find latest [releases here](https://github.com/MozarellaMan/rust-lsp-proxy/releases))
  - recommended instead of building as it's quicker
  - (*recommended*) extract and run .exe in a terminal OR add unzipped folder to PATH
  - Linux and Windows binaries available, for Mac you will need to build it yourself for now. (Instructions below)

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
    -d, --custom-lang-server-cmd

    -l, --language
    -p, --port [default: 8001]
```

- `codebase-path` is the path to the directory you want the language server to run on
- `lang-server-path` is the path to the language server binary (currently only Java supported)
- `language` is a value that tells the proxy what language is being used (refer to [this table](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocumentItem))
- `port` is the port that the proxy should listen for requests on
- `custom-lang-server-cmd` allows you to specify a custom command for the proxy to run while in the directory of your language server (allows the use of language servers not officially implemented by me for the proxy)

### Building

#### Build Dependencies

- [Rust](https://www.rust-lang.org/learn/get-started) (this will also install Cargo, the Rust build tool)
  - At *least* v1.46

#### Build Instructions

- `cargo run` will build and run the proxy in debug mode
- `cargo run --` will build and run the proxy in debug mode, with program arguments added in after the `--` (e.g. `cargo run -- -h`)
- `cargo test` will run the test suite
- `cargo build --release` will build a binary in release mode that you can find in `/target`
