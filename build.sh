set -euo pipefail

if [ -f "./Cargo.toml" ]; then
    cargo build --release
    cargo build --release --target armv7-unknown-linux-gnueabihf
else
    echo "Cargo.toml not found. Please run this script from the root of your Rust project."
    exit 1
fi

ARM64=./target/release/mymonitor
ARM32=./target/armv7-unknown-linux-gnueabihf/release/mymonitor

if [ -f "$ARM64" ]; then
    echo "Copying ARM64 binary to current directory..."
    cp "$ARM64" ./mymonitor-rpi4
else
    echo "ARM64 binary not found. Please ensure it was built successfully."
fi

if [ -f "$ARM32" ]; then
    echo "Copying ARM32 binary to current directory..."
    cp "$ARM32" ./mymonitor-rpi3b
else
    echo "ARM32 binary not found. Please ensure it was built successfully."
fi