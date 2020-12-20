use std::ffi::{OsStr, OsString};
use std::os::windows::prelude::*;
use std::{mem, ptr};
use winapi::um::libloaderapi::GetModuleHandleA;
use zoom_sdk_windows_sys as ffi;

pub fn zoom_version() -> String {
    unsafe {
        let version = ffi::ZOOMSDK_GetVersion();
        let version = u16_ptr_to_string(version);
        version.into_string().unwrap()
    }
}

pub fn init() {
    unsafe {
        let web_domain = str_to_u16_vec("https://zoom.us");
        let support_url = str_to_u16_vec("https://zoom.us");
        let mut param = ffi::ZOOMSDK_InitParam {
            strWebDomain: &web_domain[0],
            strBrandingName: ptr::null(),
            strSupportUrl: &support_url[0],
            hResInstance: GetModuleHandleA(ptr::null()) as _,
            uiWindowIconSmallID: 0,
            uiWindowIconBigID: 0,
            emLanguageID: ffi::ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_English,
            enableGenerateDump: true,
            enableLogByDefault: true,
            uiLogFileSize: 0,
            obConfigOpts: mem::zeroed(),
            locale: ffi::ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_English,
            permonitor_awareness_mode: false,
            renderOpts: mem::zeroed(),
            rawdataOpts: mem::zeroed(),
        };
        let err = ffi::ZOOMSDK_InitSDK(&mut param);
        assert!(err == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
        let err = ffi::ZOOMSDK_CleanUPSDK();
        assert!(err == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
    }
}

unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    OsString::from_wide(slice)
}

unsafe fn str_to_u16_vec(s: &str) -> Vec<u16> {
    let mut os = OsString::with_capacity(s.len());
    os.push(s);
    os.push("\0");
    os.encode_wide().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_version_equals() {
        let version = zoom_version();
        assert_eq!("5.2.1 (42037.1112)", &version);
    }

    #[test]
    fn zoom_init() {
        init();
    }
}
