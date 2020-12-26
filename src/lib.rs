//! # Zoom SDK Rust Wrapper
//!
//! The Zoom C++ API [must be called](https://devforum.zoom.us/t/list-of-active-audio-users-not-received-in-callback/1397/9)
//! from the single thread that runs the Windows message loop.

use std::ffi::{c_void, OsString};
use std::os::windows::prelude::*;
use std::panic::catch_unwind;
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
        param.strWebDomain = web_domain.as_ptr();
        param.strBrandingName = ptr::null();
        param.strSupportUrl = support_url.as_ptr();
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

static mut STATUS_CALLBACK: Option<Box<dyn Fn(&str)>> = None;

pub fn set_init_status_callback(f: impl Fn(&str) + 'static) {
    // TODO: Very very unsafe
    // let x = Box::<dyn Fn(&str)>::new(f);
    // unsafe { STATUS_CALLBACK = Some(std::mem::transmute(x)) };
    unsafe { STATUS_CALLBACK = Some(Box::new(f)) };
}

unsafe fn invoke_init_status_callback(text: &str) {
    STATUS_CALLBACK.as_ref().map(|f| f(text));
}

unsafe fn cleanup() {
    let err = ffi::ZOOMSDK_CleanUPSDK();
    assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
}

unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
    if ptr.is_null() {
        return OsString::new();
    }
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
static mut AUTH_SERVICE: *mut ffi::ZOOMSDK_IAuthService = ptr::null_mut();

unsafe fn try_auth() -> i32 {
    let err = ffi::ZOOMSDK_CreateAuthService(&mut AUTH_SERVICE);
    assert_eq!(err, ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS);
    let callback_data = Box::into_raw(Box::new(144));
    let event = ffi::ZOOMSDK_CAuthServiceEvent {
        callbackData: callback_data as _,
        authenticationReturn: Some(on_authentication_return),
        loginReturn: Some(on_login_return),
    };
    let err = ffi::ZOOMSDK_IAuthService_SetEvent(AUTH_SERVICE, &event);
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
    let err = ffi::ZOOMSDK_IAuthService_SDKAuthParam(AUTH_SERVICE, param);
    err
}

unsafe extern "C" fn on_authentication_return(data: *mut c_void, res: ffi::ZOOMSDK_AuthResult) {
    catch_unwind(|| {
        let data = data as *mut i32;
        dbg!(*data, res);
        ON_AUTH = true;
        if res == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
            let mut meeting_service = ptr::null_mut();
            let err = ffi::ZOOMSDK_CreateMeetingService(&mut meeting_service);
            dbg!(err);

            // Login
            let username = str_to_u16_vec(&std::env::var("ZOOM_LOGIN_USER").unwrap());
            let password = str_to_u16_vec(&std::env::var("ZOOM_LOGIN_PASS").unwrap());
            let param = ffi::ZOOMSDK_LoginParam {
                loginType: ffi::ZOOMSDK_LoginType_LoginType_Email,
                ut: ffi::ZOOMSDK_tagLoginParam__bindgen_ty_1 {
                    emailLogin: ffi::ZOOMSDK_tagLoginParam4Email {
                        bRememberMe: true,
                        userName: username.as_ptr(),
                        password: password.as_ptr(),
                    },
                },
            };
            dbg!(AUTH_SERVICE.is_null());
            let err = ffi::ZOOMSDK_IAuthService_Login(AUTH_SERVICE, param);
            dbg!(err);
            invoke_init_status_callback("SDK Authenticated");
        } else {
            invoke_init_status_callback("SDK Authentication failed");
        }
    });
}

unsafe extern "C" fn on_login_return(
    data: *mut c_void,
    ret: ffi::ZOOMSDK_LOGINSTATUS,
    info: *mut ffi::ZOOMSDK_IAccountInfo,
) {
    dbg!(ret);
    if ret == ffi::ZOOMSDK_LOGINSTATUS_LOGIN_SUCCESS {
        invoke_init_status_callback("Logged in");
        let display_name = ffi::ZOOMSDK_IAccountInfo_GetDisplayName(info);
        dbg!(u16_ptr_to_string(display_name));
        // ffi::ZOOMSDK_IAccountInfo_Drop(info); Should not be dropped apparently
    }
}

unsafe fn dbg_error() {
    let err_type = ffi::ZOOMSDK_GetZoomLastError();
    dbg!(err_type);
    if !err_type.is_null() {
        dbg!(ffi::ZOOMSDK_IZoomLastError_GetErrorType(err_type));
        dbg!(ffi::ZOOMSDK_IZoomLastError_GetErrorCode(err_type));
        let description = ffi::ZOOMSDK_IZoomLastError_GetErrorDescription(err_type);
        dbg!(u16_ptr_to_string(description));
        // const pointers returned so don't need drop (demo\sdk_demo_v2\mess_info.cpp)
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
}
