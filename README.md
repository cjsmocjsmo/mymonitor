# mymonitor

Lightweight Rust system monitor with two runtime modes:

- `-u`: local terminal UI dashboard (no websocket streaming)
- `-s`: websocket streaming server (no UI)

To be used in conjunction with monitorhub

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
cargo build --release --target armv7-unknown-linux-gnueabihf
```

Binary path:

```text
target/release/mymonitor
target/armv7-unknown-linux-gnueabihf/release/mymonitor
```

## Install

Since this is primarily for the raspberry pi series
I cross-compile the 32 bit binary on a raspberry pi 4

The script will detect which arch you are on and intall 
the correct binaries and the correct service file, and use 
systemctl to enable and start the service.

```bash
sh ./install.sh
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
For remote clients, connect using the machine IP, for example:

```text
ws://192.168.1.50:9001
```

## Connect With wscat

```bash
wscat -c ws://127.0.0.1:9001
```

You should see JSON metric snapshots streaming continuously.
