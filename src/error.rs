use crate::ffi;
use std::borrow::Cow;
use std::fmt;

pub(crate) type ZoomResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    err_type: ErrorType,
    message: Cow<'static, str>,
    detail: Option<ErrorDetail>,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("zoom_sdk::Error");
        s.field("type", &self.err_type)
            .field("message", &self.message);
        if let Some(detail) = &self.detail {
            s.field("detail", detail);
        }
        s.finish()
    }
}

pub(crate) trait ErrorExt {
    fn err_wrap(self, with_detail: bool) -> ZoomResult<()>;
}

impl ErrorExt for i32 {
    fn err_wrap(self, with_detail: bool) -> ZoomResult<()> {
        if self == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
            Ok(())
        } else {
            Err(Error::new(self, with_detail))
        }
    }
}

impl Error {
    pub(crate) fn new(err_type: i32, with_detail: bool) -> Self {
        let (err_type, message) = map_err_type(err_type);
        let detail = if with_detail { get_last_error() } else { None };
        Error {
            err_type,
            message,
            detail,
        }
    }

    pub(crate) fn new_rust(message: impl Into<Cow<'static, str>>) -> Self {
        Error {
            err_type: ErrorType::Rust,
            message: message.into(),
            detail: None,
        }
    }
}

#[derive(Debug)]
struct ErrorDetail {
    category: &'static str,
    /// <https://marketplace.zoom.us/docs/sdk/native-sdks/windows/resource/error-codes>
    code: u64,
    description: String,
}

fn get_last_error() -> Option<ErrorDetail> {
    unsafe {
        let last_err = ffi::ZOOMSDK_GetZoomLastError();
        if !last_err.is_null() {
            let category = match ffi::ZOOMSDK_IZoomLastError_GetErrorType(last_err) {
                ffi::ZOOMSDK_LastErrorType_LastErrorType_None => "No error",
                ffi::ZOOMSDK_LastErrorType_LastErrorType_Auth => "Auth (verification)",
                ffi::ZOOMSDK_LastErrorType_LastErrorType_Login => "Login",
                ffi::ZOOMSDK_LastErrorType_LastErrorType_Meeting => "Meeting (associated error)",
                ffi::ZOOMSDK_LastErrorType_LastErrorType_System => {
                    "System (associated error with SDK bottom layer)"
                }
                _ => "Unknown LastErrorType",
            };
            let code = ffi::ZOOMSDK_IZoomLastError_GetErrorCode(last_err);
            let description =
                crate::u16_to_string(ffi::ZOOMSDK_IZoomLastError_GetErrorDescription(last_err));
            // const pointers returned so don't need drop (demo\sdk_demo_v2\mess_info.cpp)
            return Some(ErrorDetail {
                category,
                code,
                description,
            });
        }
    }
    None
}

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    Success,
    NoImpl,
    WrongUsage,
    InvalidParameter,
    ModuleLoadFailed,
    MemoryFailed,
    ServiceFailed,
    Uninitialized,
    Unauthorized,
    NoRecordingInProgress,
    TranscoderNotFound,
    VideoNotReady,
    NoPermission,
    Unknown,
    OtherSdkInstanceRunning,
    InternalError,
    NoAudioDeviceIsFound,
    NoVideoDeviceIsFound,
    TooFrequentCall,
    FailAssignUserPrivilege,
    MeetingNotSupportedFeature,
    MeetingNotShareSender,
    MeetingYouHaveNoShare,
    MeetingViewTypeParameterIsWrong,
    MeetingAnnotationIsOff,
    SettingOsNotSupported,
    EmailLoginIsDisabled,
    HardwareNotMetForVb,
    Undocumented,
    Rust,
}

fn map_err_type(err_type: i32) -> (ErrorType, Cow<'static, str>) {
    match err_type {
        ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS => (ErrorType::Success, "Success".into()),
        ffi::ZOOMSDK_SDKError_SDKERR_NO_IMPL => (
            ErrorType::NoImpl,
            "This feature is currently invalid".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_WRONG_USEAGE => (
            ErrorType::WrongUsage,
            "Incorrect usage of the feature".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_INVALID_PARAMETER => {
            (ErrorType::InvalidParameter, "Wrong parameter".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_MODULE_LOAD_FAILED => {
            (ErrorType::ModuleLoadFailed, "Loading module failed".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_MEMORY_FAILED => {
            (ErrorType::MemoryFailed, "No memory is allocated".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_SERVICE_FAILED => {
            (ErrorType::ServiceFailed, "Internal service error".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_UNINITIALIZE => (
            ErrorType::Uninitialized,
            "Not initialized before the usage".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_UNAUTHENTICATION => (
            ErrorType::Unauthorized,
            "Not authorized before the usage".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_NORECORDINGINPROCESS => (
            ErrorType::NoRecordingInProgress,
            "No recording in process".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_TRANSCODER_NOFOUND => (
            ErrorType::TranscoderNotFound,
            "Transcoder module is not found".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_VIDEO_NOTREADY => (
            ErrorType::VideoNotReady,
            "The video service is not ready".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_NO_PERMISSION => {
            (ErrorType::NoPermission, "No permission".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_UNKNOWN => (ErrorType::Unknown, "Unknown error".into()),
        ffi::ZOOMSDK_SDKError_SDKERR_OTHER_SDK_INSTANCE_RUNNING => (
            ErrorType::OtherSdkInstanceRunning,
            "The other instance of the SDK is in process".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_INTELNAL_ERROR => {
            (ErrorType::InternalError, "SDK internal error".into())
        }
        ffi::ZOOMSDK_SDKError_SDKERR_NO_AUDIODEVICE_ISFOUND => (
            ErrorType::NoAudioDeviceIsFound,
            "No audio device found".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_NO_VIDEODEVICE_ISFOUND => (
            ErrorType::NoVideoDeviceIsFound,
            "No video device found".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_TOO_FREQUENT_CALL => (
            ErrorType::TooFrequentCall,
            "API calls too frequently".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_FAIL_ASSIGN_USER_PRIVILEGE => (
            ErrorType::FailAssignUserPrivilege,
            "<User can't be assigned with new privilege".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_DONT_SUPPORT_FEATURE => (
            ErrorType::MeetingNotSupportedFeature,
            "The current meeting doesn't support the feature".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_NOT_SHARE_SENDER => (
            ErrorType::MeetingNotShareSender,
            "The current user is not the presenter".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_YOU_HAVE_NO_SHARE => (
            ErrorType::MeetingYouHaveNoShare,
            "There is no sharing".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_VIEWTYPE_PARAMETER_IS_WRONG => (
            ErrorType::MeetingViewTypeParameterIsWrong,
            "Incorrect ViewType parameters".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_ANNOTATION_IS_OFF => (
            ErrorType::MeetingAnnotationIsOff,
            "Annotation is disabled".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_SETTING_OS_DONT_SUPPORT => (
            ErrorType::SettingOsNotSupported,
            "Current OS doesn't support the setting".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_EMAIL_LOGIN_IS_DISABLED => (
            ErrorType::EmailLoginIsDisabled,
            "Email login is disabled".into(),
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_HARDWARE_NOT_MEET_FOR_VB => (
            ErrorType::HardwareNotMetForVb,
            "Computer doesn't meet the minimum requirements to use virtual background feature"
                .into(),
        ),
        _ => (
            ErrorType::Undocumented,
            format!("Undocumented SDKError {}", err_type).into(),
        ),
    }
}
