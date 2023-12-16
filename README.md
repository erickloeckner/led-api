# led-api
Control addressable LEDs from a web-based frontend using a linux SBC (such as a Raspberry Pi) or PC with one or more SPI interface/s.

## Setup
Install the latest Rust toolchain:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repo:
```
git clone https://github.com/erickloeckner/led-api
```

Move into the cloned directory and compile the main binary:
```
cd led-api
cargo build --release
```

To compile the WebAssembly code, install wasm-pack, then move into the wasm directory and compile:
```
cargo install wasm-pack
cd wasm
wasm-pack build --release --target web
```

## Raspberry Pi-specific setup
To enable the SPI interfaces on a RPi, first enter its configuration tool:
```
sudo raspi-config
```
Select "Interface Options" from the main menu, then the SPI option, and choose "Yes" when prompted to enable the interface.
You should now see 2 files when running `ls -la /dev/spidev*` named /dev/spidev0.0 and /dev/spidev0.1.

## Building
Within the project directory, first build the main crate:
```
cargo build --release
```

Then `cd` into the wasm directory and build the WebAssembly binary:
```
cd wasm/
wasm-pack build --release --target web
```
