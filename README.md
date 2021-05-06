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
1. Immutable callbacks for events

[C++ SDK Reference]: https://marketplace.zoom.us/docs/sdk/native-sdks/windows/sdk-reference

Features:

- [ ] Initialize and cleanup SDK

## Disclaimer

The project maintainer is not affiliated with Zoom Video Communications.
