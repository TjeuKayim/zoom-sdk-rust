# Zoom SDK Rust Wrapper

[![Crate](https://img.shields.io/crates/v/zoom-sdk.svg)](https://crates.io/crates/zoom-sdk)
[![API](https://docs.rs/zoom-sdk/badge.svg)](https://docs.rs/zoom-sdk)

Idiomatic Rust bindings to
[Zoom Windows Software Development Kit](https://github.com/zoom/zoom-sdk-windows).

Status: **Work in progress, Unstable**

## Goals

1. Stick to the struct/function names from the [C++ SDK Reference] as much as possible
   (converted function names to `snake_case`)
1. Use `Drop` trait for RAII pattern

[C++ SDK Reference]: https://marketplace.zoom.us/docs/sdk/native-sdks/windows/sdk-reference

Features:

- [x] Initialize and cleanup SDK
- [x] Join meeting with web URI

## Disclaimer

The project maintainer is not affiliated with Zoom Video Communications.
