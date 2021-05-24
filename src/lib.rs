//! # Zoom SDK Rust Wrapper
//!
//! The Zoom C++ API [must be called](https://devforum.zoom.us/t/list-of-active-audio-users-not-received-in-callback/1397/9)
//! from the single thread that runs the Windows message loop.
//! # Examples
//!
//! ```
//! fn main() -> Result<(), zoom_sdk::error::Error> {
//!     zoom_sdk::init_sdk(zoom_sdk::InitParam::new()?);
//!     Ok(())
//! }
//! ```

use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::ptr;
use winapi::shared::minwindef::HMODULE;
use zoom_sdk_windows_sys as ffi;

pub mod auth;
pub mod error;
pub mod meeting;

use auth::AuthService;
use error::{Error, ErrorExt, ZoomResult};
use meeting::MeetingService;
use std::pin::Pin;

/// Get the version of ZOOM SDK.
pub fn zoom_version() -> String {
    unsafe {
        let version = ffi::ZOOMSDK_GetSDKVersion();
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
        let param = Self {
            param: unsafe { ffi::ZoomGlue_InitParam_DefaultValue() },
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
        };
        param.web_domain(Some("https://zoom.us"))
    }

    /// Web domain, defaults to 'https://zoom.us'.
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
}

/// Initialize ZOOM SDK.
///
/// Run `clean_up_sdk` to free resources.
///
/// See [C++ `InitSDK`](https://marketplacefront.zoom.us/sdk/meeting/windows/zoom__sdk_8h.html#ad2ef730cb6a637dc46747d0f2dc83893)
pub fn init_sdk(init_param: &InitParam) -> ZoomResult<()> {
    // Safety: InitParam& won't be mutated
    unsafe { ffi::ZOOMSDK_InitSDK(&init_param.param as *const _ as *mut _) }.err_wrap(true)?;
    Ok(())
}

/// Clean up ZOOM SDK.
///
/// See [C++ `CleanUPSDK`](https://marketplacefront.zoom.us/sdk/meeting/windows/zoom__sdk_8h.html#a4d51ce7c15c3ca14851acaad646d3de9).
pub fn clean_up_sdk() -> ZoomResult<()> {
    unsafe { ffi::ZOOMSDK_CleanUPSDK().err_wrap(true) }
}

/// Create authentication service interface.
/// Destroy is called automatically on drop.
pub fn create_auth_service<'a>() -> ZoomResult<Pin<Box<AuthService<'a>>>> {
    AuthService::new()
}

/// Create meeting service interface.
/// Destroy is called automatically on drop.
pub fn create_meeting_service<'a>() -> ZoomResult<MeetingService<'a>> {
    MeetingService::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_version_equals() {
        let version = zoom_version();
        assert_eq!("5.5.1 (12511.0422)", &version);
    }

    #[test]
    fn zoom_init_again() {
        // Run clean up before initialize
        clean_up_sdk().unwrap();
        clean_up_sdk().unwrap();
        // Version 5.2.1 failed this tests.
        // SDK can be initialized and cleaned up multiple times,
        // but can't be initialized second time after clean up ran once.
        // STATUS_ACCESS_VIOLATION was thrown.
        // So it might not be intended to run init multiple times.
        // Since version 5.4.3 this was fixed.
        let init_param = InitParam::new();
        init_sdk(&init_param).unwrap();
        create_auth_service().unwrap();
        // init can be called again
        init_sdk(&init_param).unwrap();
        clean_up_sdk().unwrap();
        // SDK 2
        init_sdk(&init_param).unwrap();
        create_auth_service().unwrap();
        clean_up_sdk().unwrap();
        create_auth_service().unwrap_err();
    }

    #[test]
    fn invalid_init_param() {
        let init_param = InitParam::new().web_domain(None);
        let err = init_sdk(&init_param).unwrap_err();
        assert_eq!(
            &format!("{}", err),
            r#"zoom_sdk::Error { type: InvalidParameter, message: "Wrong parameter" }"#
        );
    }
}
