use crate::{error::ErrorExt, ffi, meeting::MeetingService};
use std::marker::PhantomData;
use std::ptr;
use std::ptr::NonNull;
use winapi::shared::windef::HWND;

pub struct MeetingUiController<'a> {
    inner: NonNull<ffi::ZOOMSDK_IMeetingUIController>,
    // In the official demo this class is never freed, so it is owned by MeetingService
    phantom: PhantomData<MeetingService<'a>>,
}

impl MeetingUiController<'_> {
    pub fn new(inner: NonNull<ffi::ZOOMSDK_IMeetingUIController>) -> Self {
        MeetingUiController {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn dbg_change_float(&self, x: i32) {
        unsafe {
            ffi::ZoomGlue_IMeetingUIController_ChangeFloatoActiveSpkVideoSize(
                self.inner.as_ptr(),
                x, // ffi::ZOOMSDK_SDKFloatVideoType_FLOATVIDEO_Minimize,
            );
        }
    }

    pub fn minimize(&self) {
        unsafe {
            let mut win_handles = [ptr::null_mut(); 2];
            if let Ok(_) = dbg!(ffi::ZoomGlue_IMeetingUIController_GetMeetingUIWnd(
                self.inner.as_ptr(),
                &mut win_handles[0],
                &mut win_handles[1],
            )
            .err_wrap(true))
            {
                if !win_handles[0].is_null() {
                    minimize_window(win_handles[0] as *mut _);
                }
            }
        }
    }

    pub fn dbg_switch_minimize(&self, x: i32) {
        unsafe {
            ffi::ZoomGlue_IMeetingUIController_SwitchMinimizeUIMode4FristScreenMeetingUIWnd(
                self.inner.as_ptr(),
                x,
                // ffi::ZOOMSDK_SDKMinimizeUIMode_MinimizeUIMode_SHARE,
            );
        }
    }

    // TODO: get windows handle to minimize GetMeetingUIWnd
}

pub fn minimize_window(handle: HWND) {
    use winapi::um::winuser::{ShowWindow, SW_MINIMIZE};
    unsafe {
        ShowWindow(handle, SW_MINIMIZE);
    }
}
