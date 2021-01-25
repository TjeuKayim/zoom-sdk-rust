use crate::{ffi, str_to_u16_vec, u16_to_string, Error, ErrorExt, ZoomResult};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::catch_unwind;
use std::ptr::NonNull;
use std::{fmt, mem, ptr};

/// Authentication Service
pub struct AuthService<'a> {
    // This struct is not supposed to be Send nor Sync
    inner: NonNull<ffi::ZOOMSDK_IAuthService>,
    data: Data<'a>,
}

enum Data<'a> {
    Boxed {
        events: Option<Box<AuthService<'a>>>,
    },
    Inline {
        events: AuthServiceEvent<'a>,
        object: ffi::ZOOMSDK_AuthServiceEvent,
    },
}

pub struct AuthServiceEvent<'a> {
    // TODO: Use generic type param instead of dyn here
    //       or make this a trait.
    // TODO: How to handle errors?
    pub authentication_return: Box<dyn Fn(&AuthService, AuthResult) + 'a>,
    pub login_return: Box<dyn Fn(&AuthService, LoginStatus) + 'a>,
}

impl Drop for AuthService<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ZOOMSDK_DestroyAuthService(self.inner.as_ptr()) }
            .err_wrap(true)
            .unwrap();
    }
}

impl fmt::Debug for AuthServiceEvent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("zoom_sdk::AuthServiceEvent").finish()
    }
}

impl<'a> AuthService<'a> {
    pub(crate) fn new() -> ZoomResult<Self> {
        let mut service = ptr::null_mut();
        unsafe { ffi::ZOOMSDK_CreateAuthService(&mut service) }.err_wrap(true)?;
        if let Some(inner) = NonNull::new(service) {
            Ok(AuthService {
                inner,
                data: Data::Boxed { events: None },
            })
        } else {
            Err(Error::new_rust("ZOOMSDK_CreateAuthService returned null"))
        }
    }

    pub fn sdk_auth(&self) -> ZoomResult<()> {
        let app_key = std::env::var("ZOOM_SDK_KEY").unwrap();
        let app_key = str_to_u16_vec(&app_key);
        let app_secret = std::env::var("ZOOM_SDK_SECRET").unwrap();
        let app_secret = str_to_u16_vec(&app_secret);
        let param = ffi::ZOOMSDK_AuthParam {
            appKey: &app_key[0],
            appSecret: &app_secret[0],
        };
        unsafe { ffi::ZOOMSDK_IAuthService_SDKAuthParam(self.inner.as_ptr(), param) }
            .err_wrap(true)?;
        Ok(())
    }

    pub fn login(&self, username: &str, password: &str, remember_me: bool) -> ZoomResult<()> {
        let username = str_to_u16_vec(username);
        let password = str_to_u16_vec(password);
        let param = ffi::ZOOMSDK_LoginParam {
            loginType: ffi::ZOOMSDK_LoginType_LoginType_Email,
            ut: ffi::ZOOMSDK_tagLoginParam__bindgen_ty_1 {
                emailLogin: ffi::ZOOMSDK_tagLoginParam4Email {
                    bRememberMe: remember_me,
                    userName: username.as_ptr(),
                    password: password.as_ptr(),
                },
            },
        };
        unsafe { ffi::ZOOMSDK_IAuthService_Login(self.inner.as_ptr(), param) }.err_wrap(true)?;
        Ok(())
    }

    pub fn set_event(&mut self, events: AuthServiceEvent<'a>) -> ZoomResult<()> {
        match &mut self.data {
            Data::Inline { events: e, .. } => {
                *e = events;
                new_object(self)?;
            }
            Data::Boxed { events: e } => {
                let mut b = Box::new(AuthService {
                    inner: self.inner,
                    data: Data::Inline {
                        events,
                        object: unsafe { mem::zeroed() },
                    },
                });
                new_object(&mut *b)?;
                *e = Some(b);
            }
        };
        fn new_object(callback_data: &mut AuthService) -> ZoomResult<()> {
            let callback_data_p = callback_data as *mut _ as *mut c_void;
            if let Data::Inline { object, .. } = &mut callback_data.data {
                unsafe {
                    ffi::ZOOMSDK_AuthServiceEvent_New(object);
                    object.event = ffi::ZOOMSDK_CAuthServiceEvent {
                        callbackData: callback_data_p,
                        authenticationReturn: Some(on_authentication_return),
                        loginReturn: Some(on_login_return),
                    };
                    ffi::ZOOMSDK_IAuthService_SetEvent(
                        callback_data.inner.as_ptr(),
                        &mut object._base,
                    )
                    .err_wrap(true)?;
                }
            } else {
                panic!("not inline");
            };
            Ok(())
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AuthResult {
    /// Authentication is successful.
    Success,
    /// The key or secret to authenticate is empty.
    KeyOrSecretEmpty,
    /// The key or secret to authenticate is wrong.
    KeyOrSecretWrong,
    /// The user account does not support.
    AccountNotSupport,
    /// The user account is not enabled for SDK.
    AccountNotEnableSdk,
    /// Unknown error.
    Unknown,
    /// Service is busy.
    ServiceBuzy,
    /// Initial status.
    None,
    /// Time out.
    OverTime,
    /// Network issues.
    NetworkIssue,
    /// Account does not support this SDK version.
    ClientIncompatible,
    /// Unmapped.
    Unmapped(i32),
}

impl fmt::Display for AuthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(map_auth_result_description(*self))
    }
}

fn map_auth_result(result: i32) -> AuthResult {
    match result {
        ffi::ZOOMSDK_AuthResult_AUTHRET_SUCCESS => AuthResult::Success,
        ffi::ZOOMSDK_AuthResult_AUTHRET_KEYORSECRETEMPTY => AuthResult::KeyOrSecretEmpty,
        ffi::ZOOMSDK_AuthResult_AUTHRET_KEYORSECRETWRONG => AuthResult::KeyOrSecretWrong,
        ffi::ZOOMSDK_AuthResult_AUTHRET_ACCOUNTNOTSUPPORT => AuthResult::AccountNotSupport,
        ffi::ZOOMSDK_AuthResult_AUTHRET_ACCOUNTNOTENABLESDK => AuthResult::AccountNotEnableSdk,
        ffi::ZOOMSDK_AuthResult_AUTHRET_UNKNOWN => AuthResult::Unknown,
        ffi::ZOOMSDK_AuthResult_AUTHRET_SERVICE_BUSY => AuthResult::ServiceBuzy,
        ffi::ZOOMSDK_AuthResult_AUTHRET_NONE => AuthResult::None,
        ffi::ZOOMSDK_AuthResult_AUTHRET_OVERTIME => AuthResult::OverTime,
        ffi::ZOOMSDK_AuthResult_AUTHRET_NETWORKISSUE => AuthResult::NetworkIssue,
        ffi::ZOOMSDK_AuthResult_AUTHRET_CLIENT_INCOMPATIBLE => AuthResult::ClientIncompatible,
        _ => AuthResult::Unmapped(result),
    }
}

fn map_auth_result_description(result: AuthResult) -> &'static str {
    match result {
        AuthResult::Success => "Authentication is successful",
        AuthResult::KeyOrSecretEmpty => "The key or secret to authenticate is empty",
        AuthResult::KeyOrSecretWrong => "The key or secret to authenticate is wrong",
        AuthResult::AccountNotSupport => "The user account does not support",
        AuthResult::AccountNotEnableSdk => "The user account is not enabled for SDK",
        AuthResult::Unknown => "Unknown error",
        AuthResult::ServiceBuzy => "Service is busy",
        AuthResult::None => "Initial status",
        AuthResult::OverTime => "Time out",
        AuthResult::NetworkIssue => "Network issues",
        AuthResult::ClientIncompatible => "Account does not support this SDK version",
        _ => "Unknown AuthResult",
    }
}

#[derive(Debug)]
pub enum LoginStatus<'a> {
    /// Not logged in.
    Idle,
    /// In process of login.
    Processing,
    /// Login successful.
    Success(AccountInfo<'a>),
    /// Login failed.
    Failed,
    /// Unmapped.
    Unmapped(i32),
}

#[derive(Debug)]
pub struct AccountInfo<'a> {
    raw: NonNull<ffi::ZOOMSDK_IAccountInfo>,
    // IAccountInfo should not be dropped apparently, but is only valid for in the callback
    phantom: PhantomData<&'a AuthService<'a>>,
}

impl<'a> AccountInfo<'a> {
    fn new(raw: *mut ffi::ZOOMSDK_IAccountInfo, _lifetime: &'a ()) -> Self {
        AccountInfo {
            raw: NonNull::new(raw).expect("IAccountInfo null"),
            phantom: PhantomData,
        }
    }

    pub fn get_display_name(&self) -> String {
        unsafe { u16_to_string(ffi::ZOOMSDK_IAccountInfo_GetDisplayName(self.raw.as_ptr())) }
    }
    // TODO: GetLoginType
}

unsafe extern "C" fn on_authentication_return(data: *mut c_void, res: ffi::ZOOMSDK_AuthResult) {
    let _ = catch_unwind(|| {
        let service = &mut *(data as *mut AuthService);
        if let Data::Inline { events, .. } = &service.data {
            (events.authentication_return)(service, map_auth_result(res));
        }
    });
    // if res == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
    //     let mut meeting_service = ptr::null_mut();
    //     let err = ffi::ZOOMSDK_CreateMeetingService(&mut meeting_service);
    //     dbg!(err);
    //
    //     invoke_init_status_callback("SDK Authenticated");
    // } else {
    //     invoke_init_status_callback("SDK Authentication failed");
    // }
}

unsafe extern "C" fn on_login_return(
    data: *mut c_void,
    ret: ffi::ZOOMSDK_LOGINSTATUS,
    info: *mut ffi::ZOOMSDK_IAccountInfo,
) {
    let lifetime = ();
    let _ = catch_unwind(|| {
        let status = match ret {
            ffi::ZOOMSDK_LOGINSTATUS_LOGIN_IDLE => LoginStatus::Idle,
            ffi::ZOOMSDK_LOGINSTATUS_LOGIN_PROCESSING => LoginStatus::Processing,
            ffi::ZOOMSDK_LOGINSTATUS_LOGIN_SUCCESS => {
                LoginStatus::Success(AccountInfo::new(info, &lifetime))
            }
            ffi::ZOOMSDK_LOGINSTATUS_LOGIN_FAILED => LoginStatus::Failed,
            _ => LoginStatus::Unmapped(ret),
        };
        let service = &mut *(data as *mut AuthService);
        if let Data::Inline { events, .. } = &service.data {
            (events.login_return)(service, status);
        }
    });
    // dbg!(ret);
    // if ret == ffi::ZOOMSDK_LOGINSTATUS_LOGIN_SUCCESS {
    //     invoke_init_status_callback("Logged in");
    //     let display_name = ffi::ZOOMSDK_IAccountInfo_GetDisplayName(info);
    //     dbg!(u16_ptr_to_os_string(display_name));
}
