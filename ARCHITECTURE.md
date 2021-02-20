# Architecture

This document describes the high level architecture of the Language Server Proxy, this is the way to easily get up to speed with what the code is doing.

## Birds Eye View

[image here]

At the highest level, the proxy is simply taking messages from a websocket connection that is initialized with a GET request to the `/ls` endopoint, and sending those messages to a Language Server Protocol compliant server child process. Some messages are intercepted to provide file synchronization client edits to the proxy's files. Files are requested with GET requests.

## Endpoints

`/ls`
A GET request to this endpoint initialises the websocket connection to the language server
