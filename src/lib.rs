//! # Zoom SDK Rust Wrapper
//!
//! The Zoom C++ API [must be called](https://devforum.zoom.us/t/list-of-active-audio-users-not-received-in-callback/1397/9)
//! from the single thread that runs the Windows message loop.

use std::ffi::{c_void, OsString};
use std::os::windows::prelude::*;
use std::panic::catch_unwind;
use std::{fmt, mem, ptr};
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::GetModuleHandleA;
use zoom_sdk_windows_sys as ffi;

use error::{Error, ErrorExt, ZoomResult};

mod error;

pub fn zoom_version() -> String {
    unsafe {
        let version = ffi::ZOOMSDK_GetVersion();
        let version = u16_ptr_to_os_string(version);
        version.into_string().unwrap()
    }
}

/// Builder to initialize the SDK parameter.
///
/// [C++ InitParam](https://marketplacefront.zoom.us/sdk/meeting/windows/structtag_init_param.html)
#[derive(Clone, Debug)]
pub struct InitParam {
    param: ffi::ZOOMSDK_InitParam,
    // Cache for encoded strings
    web_domain: Option<Vec<u16>>,
    branding_name: Option<Vec<u16>>,
    support_url: Option<Vec<u16>>,
}

impl InitParam {
    /// Creates default builder for InitParam.
    pub fn new() -> Self {
        Self {
            param: unsafe { ffi::ZOOMSDK_InitParam_Default() },
            // string_cache: StringCache(Vec::new()),
            web_domain: None,
            branding_name: None,
            support_url: None,
            // res_instance: -1isize as usize as HMODULE,
            // ui_window_icon_small_id: 0,
            // ui_window_icon_big_id: 0,
            // em_language_id: SdkLanguageId::Unknown,
            // enable_generate_dump: false,
            // enable_log_by_default: false,
            // ui_log_file_size: 5,
        }
    }

    /// Web domain.
    pub fn web_domain(mut self, web_domain: Option<&str>) -> Self {
        option_str_encode_nul_wide(
            &mut self.web_domain,
            web_domain,
            &mut self.param.strWebDomain,
        );
        self
    }

    /// Branding name.
    pub fn branding_name(mut self, branding_name: Option<&str>) -> Self {
        option_str_encode_nul_wide(
            &mut self.branding_name,
            branding_name,
            &mut self.param.strBrandingName,
        );
        self
    }

    /// Support URL.
    pub fn support_url(mut self, support_url: Option<&str>) -> Self {
        option_str_encode_nul_wide(
            &mut self.support_url,
            support_url,
            &mut self.param.strSupportUrl,
        );
        self
    }

    /// Resource module handle.
    pub fn res_instance(mut self, res_instance: HMODULE) -> Self {
        self.param.hResInstance = res_instance as _;
        self
    }

    /// The ID of the small icon on the window.
    pub fn ui_window_icon_small_id(mut self, ui_window_icon_small_id: u32) -> Self {
        self.param.uiWindowIconSmallID = ui_window_icon_small_id;
        self
    }

    /// The ID of the big Icon on the window.
    pub fn ui_window_icon_big_id(mut self, ui_window_icon_big_id: u32) -> Self {
        self.param.uiWindowIconBigID = ui_window_icon_big_id;
        self
    }

    /// The ID of the SDK language.
    pub fn em_language_id(mut self, em_language_id: SdkLanguageId) -> Self {
        self.param.emLanguageID = em_language_id as i32;
        self
    }

    /// Enable generate dump file if the app crashed.
    pub fn enable_generate_dump(mut self, enable_generate_dump: bool) -> Self {
        self.param.enableGenerateDump = enable_generate_dump;
        self
    }

    /// Enable log feature.
    pub fn enable_log_by_default(mut self, enable_log_by_default: bool) -> Self {
        self.param.enableLogByDefault = enable_log_by_default;
        self
    }

    /// Size of a log file in M(megabyte). The default size is 5M. There are 5 log files in total and the file size varies from 1M to 50M.
    pub fn ui_log_file_size(mut self, ui_log_file_size: u32) -> Self {
        self.param.uiLogFileSize = ui_log_file_size;
        self
    }

    // TODO: ConfigOpts, locale, permonitor_awareness_mode, renderOpts, rawdataOpts

    pub fn init_sdk(mut self) -> ZoomResult<Sdk> {
        unsafe { ffi::ZOOMSDK_InitSDK(&mut self.param) }.err_wrap()?;
        // TODO: Must CleanUPSDK be called if InitSDK failed?
        Ok(Sdk {})
    }
}

/// Initialized SDK returned by [`InitParam::init_sdk`].
/// Drop runs [C++ CleanUPSDK](https://marketplacefront.zoom.us/sdk/meeting/windows/zoom__sdk_8h.html#a4d51ce7c15c3ca14851acaad646d3de9).
pub struct Sdk {}

impl Sdk {
    pub fn clean_up_sdk(self) -> Result<(), (Error, Sdk)> {
        self.clean_up_internal().map_err(|e| (e, self))
    }

    fn clean_up_internal(&self) -> ZoomResult<()> {
        unsafe { ffi::ZOOMSDK_CleanUPSDK() }.err_wrap()
    }
}

impl Drop for Sdk {
    fn drop(&mut self) {
        self.clean_up_internal().unwrap();
    }
}

/// Encodes nul-terminated wide string and stores in cache.
/// # Safety
/// Cache must not be reassigned to prevent dangling pointer.
fn option_str_encode_nul_wide(
    cache: &mut Option<Vec<u16>>,
    from: Option<&str>,
    to: &mut *const u16,
) {
    *to = from.map_or(ptr::null_mut(), |s| {
        let mut vec = str_to_u16_vec(s);
        let ptr = vec.as_mut_ptr();
        *cache = Some(vec);
        ptr
    });
}

/// The text resource type used by the SDK.
/// [C++ SDK_LANGUAGE_ID](https://marketplacefront.zoom.us/sdk/meeting/windows/zoom__sdk__def_8h.html#a9747f9758092fe2d88bb5e2d45e717c5)
#[derive(Copy, Clone, Debug)]
pub enum SdkLanguageId {
    /// For initialization.
    Unknown = 0,
    /// In English.
    English,
    /// In simplified Chinese.
    ChineseSimplified,
    /// In traditional Chinese.
    ChineseTraditional,
    /// In Japanese.
    Japanese,
    /// In Spanish.
    Spanish,
    /// In German.
    German,
    /// In French.
    French,
    /// In Portuguese.
    Portuguese,
    /// In Russian.
    Russian,
    /// In Korean.
    Korean,
    /// In Vietnamese.
    Vietnamese,
    /// In Italian.
    Italian,
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

unsafe fn u16_ptr_to_os_string(ptr: *const u16) -> OsString {
    if ptr.is_null() {
        return OsString::new();
    }
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    OsString::from_wide(slice)
}

unsafe fn u16_to_string(ptr: *const u16) -> String {
    u16_ptr_to_os_string(ptr)
        .into_string()
        .unwrap_or("Invalid string encoding".to_string())
}

fn str_to_u16_vec(s: &str) -> Vec<u16> {
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
        dbg!(u16_ptr_to_os_string(display_name));
        // ffi::ZOOMSDK_IAccountInfo_Drop(info); Should not be dropped apparently
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
