The original idea for this project came from

https://github.com/Tinega-Devops/monitor-rs

I added the websocket server and the multicore support for the raspberry pi.

This project is targeted towards the Raspberry Pi 5, 4 (aarch64) , and the 3b+ (armv7l) using the built in multiarch support.


To build this project you will need these prerequsits:

    sudo apt update
    sudo apt install gcc-arm-linux-gnueabihf libc6-dev-armhf-cross

    rustup target add armv7-unknown-linux-gnueabihf

    (use bash and not sh)

    bash build.sh 0.1.3

Or you can use the prebuilt binaries:

    ./mymonitor-rpi4-0.1.2 (for 64bit arm systems)
    ./mymonitor-rpi3b-0.1.2(for 32bit arm systems)

This should run on a AMD or Intel 64 bit systems just build as usual:

    cargo run --release