use crate::{ffi, str_to_u16_vec, u16_ptr_to_os_string, Error, ErrorExt, ZoomResult};
use std::ffi::c_void;
use std::panic::catch_unwind;
use std::ptr;
use std::ptr::NonNull;

pub struct AuthService {
    inner: NonNull<ffi::ZOOMSDK_IAuthService>,
    events: Box<Events>,
}

struct Events {
    authentication_return: Box<dyn FnMut()>,
    login_return: Box<dyn FnMut()>,
}

impl Drop for AuthService {
    fn drop(&mut self) {
        unsafe { ffi::ZOOMSDK_DestroyAuthService(self.inner.as_ptr()) }
            .err_wrap(true)
            .unwrap();
    }
}

impl AuthService {
    pub(crate) fn new() -> ZoomResult<Self> {
        let mut service = ptr::null_mut();
        unsafe { ffi::ZOOMSDK_CreateAuthService(&mut service) }.err_wrap(true)?;
        if let Some(inner) = NonNull::new(service) {
            let events = set_event(inner)?;
            Ok(AuthService { inner, events })
        } else {
            Err(Error::new_rust("ZOOMSDK_CreateAuthService returned null"))
        }
    }

    pub fn sdk_auth(&mut self) -> ZoomResult<()> {
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

    pub fn login(&mut self) -> ZoomResult<()> {
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
        unsafe { ffi::ZOOMSDK_IAuthService_Login(self.inner.as_ptr(), param) }.err_wrap(true)?;
        Ok(())
    }
}

fn set_event(inner: NonNull<ffi::ZOOMSDK_IAuthService>) -> ZoomResult<Box<Events>> {
    let events = Box::new(Events {
        authentication_return: Box::new(|| println!("auth ret")),
        login_return: Box::new(|| {}),
    });
    let callback_data = Box::into_raw(events);
    let events = unsafe { Box::from_raw(callback_data) };
    let c_event = ffi::ZOOMSDK_CAuthServiceEvent {
        callbackData: callback_data as _,
        authenticationReturn: Some(on_authentication_return),
        loginReturn: Some(on_login_return),
    };
    unsafe { ffi::ZOOMSDK_IAuthService_SetEvent(inner.as_ptr(), &c_event).err_wrap(true)? };
    Ok(events)
}

unsafe extern "C" fn on_authentication_return(data: *mut c_void, res: ffi::ZOOMSDK_AuthResult) {
    let _ = catch_unwind(|| {
        let events = &mut *(data as *mut Events);
        (events.authentication_return)();
        // if res == ffi::ZOOMSDK_SDKError_SDKERR_SUCCESS {
        //     let mut meeting_service = ptr::null_mut();
        //     let err = ffi::ZOOMSDK_CreateMeetingService(&mut meeting_service);
        //     dbg!(err);
        //
        //     invoke_init_status_callback("SDK Authenticated");
        // } else {
        //     invoke_init_status_callback("SDK Authentication failed");
        // }
    });
}

unsafe extern "C" fn on_login_return(
    data: *mut c_void,
    ret: ffi::ZOOMSDK_LOGINSTATUS,
    info: *mut ffi::ZOOMSDK_IAccountInfo,
) {
    let events = &mut *(data as *mut Events);
    (events.login_return)();
    // dbg!(ret);
    // if ret == ffi::ZOOMSDK_LOGINSTATUS_LOGIN_SUCCESS {
    //     invoke_init_status_callback("Logged in");
    //     let display_name = ffi::ZOOMSDK_IAccountInfo_GetDisplayName(info);
    //     dbg!(u16_ptr_to_os_string(display_name));
    //     // ffi::ZOOMSDK_IAccountInfo_Drop(info); Should not be dropped apparently
    // }
}
