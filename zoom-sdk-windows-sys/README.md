# zoom-sdk-windows-sys

[![Crate](https://img.shields.io/crates/v/zoom-sdk-windows-sys.svg)](https://crates.io/crates/rand)
[![API](https://docs.rs/zoom-sdk-windows-sys/badge.svg)](https://docs.rs/rand)

FFI bindings to [Zoom Windows Software Development Kit](https://github.com/zoom/zoom-sdk-windows).

## Build steps for Windows 10

1. Visual Studio Build tools with Windows 10 SDK
1. Install 32-bit target `rustup target add i686-pc-windows-msvc`
1. [Install LLVM for bindgen](https://rust-lang.github.io/rust-bindgen/requirements.html#windows)
1. Download the [SDK from Github](https://github.com/zoom/zoom-sdk-windows/)
1. Add SDK `bin` directory with DLLs to PATH
1. Set environment variable `ZOOM_SDK_DIR=path\to\zoom-sdk-windows`

TODO: Build script
