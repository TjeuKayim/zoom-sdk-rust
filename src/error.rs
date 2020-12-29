use crate::ffi;
use std::fmt;

pub(crate) type ZoomResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    err_type: ErrorType,
    message: &'static str,
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
    fn err_wrap(self) -> ZoomResult<()>;
}

impl ErrorExt for i32 {
    fn err_wrap(self) -> ZoomResult<()> {
        if self == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
            Ok(())
        } else {
            Err(Error::new(self))
        }
    }
}

impl Error {
    pub(crate) fn new(err_type: i32) -> Self {
        let (err_type, message) = map_err_type(err_type);
        let detail = get_last_error();
        Error {
            err_type,
            message,
            detail,
        }
    }
}

#[derive(Debug)]
struct ErrorDetail {
    category: &'static str,
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
}

fn map_err_type(err_type: i32) -> (ErrorType, &'static str) {
    match err_type {
        ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS => (ErrorType::Success, "Success"),
        ffi::ZOOMSDK_SDKError_SDKERR_NO_IMPL => {
            (ErrorType::NoImpl, "This feature is currently invalid")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_WRONG_USEAGE => {
            (ErrorType::WrongUsage, "Incorrect usage of the feature")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_INVALID_PARAMETER => {
            (ErrorType::InvalidParameter, "Wrong parameter")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_MODULE_LOAD_FAILED => {
            (ErrorType::ModuleLoadFailed, "Loading module failed")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_MEMORY_FAILED => {
            (ErrorType::MemoryFailed, "No memory is allocated")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_SERVICE_FAILED => {
            (ErrorType::ServiceFailed, "Internal service error")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_UNINITIALIZE => {
            (ErrorType::Uninitialized, "Not initialized before the usage")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_UNAUTHENTICATION => {
            (ErrorType::Unauthorized, "Not authorized before the usage")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_NORECORDINGINPROCESS => {
            (ErrorType::NoRecordingInProgress, "No recording in process")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_TRANSCODER_NOFOUND => (
            ErrorType::TranscoderNotFound,
            "Transcoder module is not found",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_VIDEO_NOTREADY => {
            (ErrorType::VideoNotReady, "The video service is not ready")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_NO_PERMISSION => (ErrorType::NoPermission, "No permission"),
        ffi::ZOOMSDK_SDKError_SDKERR_UNKNOWN => (ErrorType::Unknown, "Unknown error"),
        ffi::ZOOMSDK_SDKError_SDKERR_OTHER_SDK_INSTANCE_RUNNING => (
            ErrorType::OtherSdkInstanceRunning,
            "The other instance of the SDK is in process",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_INTELNAL_ERROR => {
            (ErrorType::InternalError, "SDK internal error")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_NO_AUDIODEVICE_ISFOUND => {
            (ErrorType::NoAudioDeviceIsFound, "No audio device found")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_NO_VIDEODEVICE_ISFOUND => {
            (ErrorType::NoVideoDeviceIsFound, "No video device found")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_TOO_FREQUENT_CALL => {
            (ErrorType::TooFrequentCall, "API calls too frequently")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_FAIL_ASSIGN_USER_PRIVILEGE => (
            ErrorType::FailAssignUserPrivilege,
            "<User can't be assigned with new privilege",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_DONT_SUPPORT_FEATURE => (
            ErrorType::MeetingNotSupportedFeature,
            "The current meeting doesn't support the feature",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_NOT_SHARE_SENDER => (
            ErrorType::MeetingNotShareSender,
            "The current user is not the presenter",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_YOU_HAVE_NO_SHARE => {
            (ErrorType::MeetingYouHaveNoShare, "There is no sharing")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_VIEWTYPE_PARAMETER_IS_WRONG => (
            ErrorType::MeetingViewTypeParameterIsWrong,
            "Incorrect ViewType parameters",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_MEETING_ANNOTATION_IS_OFF => {
            (ErrorType::MeetingAnnotationIsOff, "Annotation is disabled")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_SETTING_OS_DONT_SUPPORT => (
            ErrorType::SettingOsNotSupported,
            "Current OS doesn't support the setting",
        ),
        ffi::ZOOMSDK_SDKError_SDKERR_EMAIL_LOGIN_IS_DISABLED => {
            (ErrorType::EmailLoginIsDisabled, "Email login is disabled")
        }
        ffi::ZOOMSDK_SDKError_SDKERR_HARDWARE_NOT_MEET_FOR_VB => (
            ErrorType::HardwareNotMetForVb,
            "Computer doesn't meet the minimum requirements to use virtual background feature",
        ),
        _ => (ErrorType::Undocumented, "Undocumented SDKError"),
    }
}
