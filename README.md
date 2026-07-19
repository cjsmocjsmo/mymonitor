# mymonitor

Lightweight Rust system monitor with two runtime modes:

- `-u`: local terminal UI dashboard (no websocket streaming)
- `-s`: websocket streaming server (no UI)

## Debian Packages

Install these packages on Debian/Ubuntu before building:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config curl ca-certificates git
```

### Optional: Armv7 Cross-Compile (Raspberry Pi 3 32-bit)

If you want to build `armv7-unknown-linux-gnueabihf` binaries on a non-ARM host:

```bash
sudo apt install -y gcc-arm-linux-gnueabihf libc6-dev-armhf-cross
```

## Rust Toolchain Setup

If Rust is not installed yet:

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

(Optional armv7 target):

```bash
rustup target add armv7-unknown-linux-gnueabihf
```

## Build

From the project root:

```bash
cargo build --release
```

Binary path:

```text
target/release/mymonitor
```

## Run

The program requires exactly one mode argument.

### UI Mode (`-u`)

Displays metrics in the terminal UI and does not start websocket streaming.

```bash
./target/release/mymonitor -u
```

### Streaming Mode (`-s`)

Starts websocket server only (no UI). Metrics are sent once per second as JSON.

```bash
./target/release/mymonitor -s
```

Server bind address:

```text
ws://0.0.0.0:9001
```

For remote clients, connect using the machine IP, for example:

```text
ws://192.168.1.50:9001
```

## Connect With wscat

Install Node.js + npm if needed:

```bash
sudo apt install -y nodejs npm
```

Install wscat:

```bash
npm install -g wscat
```

Connect to the running server:

```bash
wscat -c ws://127.0.0.1:9001
```

Or from another machine on the same network:

```bash
wscat -c ws://192.168.1.50:9001
```

You should see JSON metric snapshots streaming continuously.
