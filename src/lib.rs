//! # Zoom SDK Rust Wrapper
//!
//! The Zoom C++ API [must be called](https://devforum.zoom.us/t/list-of-active-audio-users-not-received-in-callback/1397/9)
//! from the single thread that runs the Windows message loop.

use std::ffi::OsString;
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

pub struct Builder {}

pub fn init() {
    unsafe {
        let web_domain = str_to_u16_vec("https://zoom.us");
        let support_url = str_to_u16_vec("https://zoom.us");
        let mut param = ffi::ZOOMSDK_InitParam_Default();
        param.strWebDomain = &web_domain[0];
        param.strBrandingName = ptr::null();
        param.strSupportUrl = &support_url[0];
        param.hResInstance = GetModuleHandleA(ptr::null()) as _;
        // param.uiWindowIconSmallID = 0;
        // param.uiWindowIconBigID = 0;
        // param.emLanguageID = ffi::ZOOMSDK_SDK_LANGUAGE_ID_LANGUAGE_Unknow;
        param.enableGenerateDump = true;
        param.enableLogByDefault = true;
        // param.uiLogFileSize = 5;
        // param.obConfigOpts = mem::zeroed();
        // param.locale = ffi::ZOOMSDK_SDK_APP_Locale_SDK_APP_Locale_Default;
        // param.permonitor_awareness_mode = true;
        // param.renderOpts = mem::zeroed();
        // param.rawdataOpts = mem::zeroed();
        dbg!(param);
        let err = ffi::ZOOMSDK_InitSDK(&mut param);
        assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
        try_auth();
    }
}

unsafe fn cleanup() {
    let err = ffi::ZOOMSDK_CleanUPSDK();
    assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
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

static mut ON_AUTH: bool = false;

unsafe fn try_auth() -> i32 {
    let mut auth_service = ptr::null_mut();
    let err = ffi::ZOOMSDK_CreateAuthService(&mut auth_service);
    assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
    let event = ffi::ZOOMSDK_AuthServiceEvent_New(Some(on_authentication_return));
    assert!(!event.is_null());
    let err = ffi::ZOOMSDK_IAuthService_SetEvent(auth_service, event);
    assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
    let app_key: Vec<u16> = std::env::var_os("ZOOM_SDK_KEY")
        .unwrap()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let app_secret: Vec<u16> = std::env::var_os("ZOOM_SDK_SECRET")
        .unwrap()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let param = ffi::ZOOMSDK_AuthParam {
        appKey: &app_key[0],
        appSecret: &app_secret[0],
    };
    return ffi::ZOOMSDK_IAuthService_SDKAuthParam(auth_service, param);
}

unsafe extern "C" fn on_authentication_return(res: ffi::ZOOMSDK_AuthResult) {
    dbg!(res);
    ON_AUTH = true;
    if res == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
        // let meeting_service = ffi::ZOOMSDK_CreateMeetingService();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::exit;

    #[test]
    fn zoom_version_equals() {
        let version = zoom_version();
        assert_eq!("5.2.1 (42037.1112)", &version);
    }

    #[test]
    fn zoom_init() {
        init();
    }

    #[test]
    fn zoom_init_err() {
        unsafe {
            init();
            std::thread::sleep(std::time::Duration::from_secs(10));
            // assert_eq!(try_auth(), ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
            // std::thread::sleep(std::time::Duration::from_secs(10));
            // dbg!(ON_AUTH);
        }
    }

    unsafe fn dbg_error() {
        let err_type = ffi::ZOOMSDK_GetZoomLastError();
        dbg!(err_type);
        if !err_type.is_null() {
            dbg!(ffi::ZOOMSDK_IZoomLastError_GetErrorType(err_type));
            dbg!(ffi::ZOOMSDK_IZoomLastError_GetErrorCode(err_type));
            dbg!(u16_ptr_to_string(
                ffi::ZOOMSDK_IZoomLastError_GetErrorDescription(err_type)
            ));
        }
    }
}
