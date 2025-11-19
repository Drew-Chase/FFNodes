# FFNodes
Have you ever wanted to encode many video or audio files at once, across multiple devices? FFNodes is a Rust-based tool designed to simplify this process by leveraging the power of ffmpeg and distributed computing.
## Table of Contents


## Features- Distributed encoding across multiple devices
- Easy-to-use command-line interface
- Support for various video and audio formats
- Customizable encoding parameters
- Real-time monitoring of encoding progress

## Installation
FFNodes is available on Windows, Linux, and MacOS, for both x86 and ARM architectures.
> **Note:** FFNodes requires ffmpeg to be installed on your system and accessible from the command line. You can find ffmpeg [here](https://ffmpeg.org/download.html).
### Server
The server is a simple REST API that can be used to manage encoding jobs. You can Install the server in a few ways:
1. Using Cargo Install: `cargo install ffnodes-server --git https://github.com/Drew-Chase/FFNodes.git`
2. Using our installer script: `curl -fsSL https://raw.githubusercontent.com/Drew-Chase/FFNodes/main/install.sh | bash` (linux only) 
3. Using our MSI installer found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (windows only)
4. Building from source: `git clone https://github.com/Drew-Chase/FFNodes.git && cd FFNodes && cargo build --release --bin ffnodes-server`.
5. Using Docker: `docker run -p 8080:8080 ghcr.io/drew-chase/ffnodes-server:latest`
6. Using Homebrew: `brew install drew-chase/tap/ffnodes-server` (macos only)
7. Using winget: `winget install FFNodes.FFNodesServer` (windows only)
### Client
The client is a GUI application that can be used to manage encoding jobs. You can install the client in a few ways:
1. Using Cargo Install: `cargo install ffnodes-client --git https://github.com/Drew-Chase/FFNodes.git`
2. Using our appimage file found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (linux only)
3. Using our MSI installer found in the [releases page](https://github.com/Drew-Chase/FFNodes/releases) (windows only)
4. Building from source: `git clone https://github.com/Drew-Chase/FFNodes.git && cd FFNodes && cargo build --release --bin ffnodes-client`.
5. Using Homebrew: `brew install drew-chase/tap/ffnodes-client` (macos only)
6. Using winget: `winget install FFNodes.FFNodesClient` (windows only)

## Technology
FFNodes is built using Rust for performance and safety.
### Server
The server uses the following technologies:
- [Actix Web](https://actix.rs/) - For the REST API
- [SQLX](https://crates.io/crates/sqlx) with SQLite - For storing job metadata and found media files
- [Tokio](https://tokio.rs/) - For asynchronous tasks
- [Notify](https://crates.io/crates/notify) - For watching for media files
### Client
The client uses the following technologies:
- [Tauri](https://tauri.app/)
- [React](https://reactjs.org/)
- [Typescript](https://www.typescriptlang.org/)
- [TailwindCSS](https://tailwindcss.com/)
- [HeroUI](https://heroui.com)
- [React Router](https://reactrouter.com/)