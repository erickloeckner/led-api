# led-api
Control addressable LEDs from a web-based frontend using a linux SBC (such as a Raspberry Pi) or PC with one or more SPI interface/s.

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
