//! Status: **Work in progress, Unstable**
//!
//! FFI bindings to [Zoom Windows Software Development Kit](https://github.com/zoom/zoom-sdk-windows)
//! generated with bindgen.
//!
//! Also contains C++ glue code for the virtual methods.
//!
//! See also the official [C++ SDK reference](https://marketplacefront.zoom.us/sdk/meeting/windows/files.html)

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::windows::prelude::*;

    #[test]
    fn zoom_get_version() {
        unsafe {
            let version = ZOOMSDK_GetVersion();
            let version = u16_ptr_to_string(version);
            dbg!(version);
        }
    }

    unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
        let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        OsString::from_wide(slice)
    }
}
