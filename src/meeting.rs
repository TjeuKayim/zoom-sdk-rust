use crate::{ffi, str_to_u16_vec, u16_to_string, Error, ErrorExt, Sdk, ZoomResult};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::catch_unwind;
use std::ptr::NonNull;
use std::{fmt, ptr};

/// Meeting Service
pub struct MeetingService<'a> {
    /// This struct is not supposed to be Send nor Sync
    inner: NonNull<ffi::ZOOMSDK_IMeetingService>,
    #[allow(dead_code)]
    sdk: &'a Sdk,
}

impl Drop for MeetingService<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ZOOMSDK_DestroyMeetingService(self.inner.as_ptr()) }
            .err_wrap(true)
            .unwrap();
    }
}

impl fmt::Debug for MeetingService<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("zoom_sdk::MeetingService").finish()
    }
}

impl<'a> MeetingService<'a> {
    pub(crate) fn new(sdk: &'a Sdk) -> ZoomResult<Self> {
        let mut service = ptr::null_mut();
        unsafe { ffi::ZOOMSDK_CreateMeetingService(&mut service) }.err_wrap(true)?;
        if let Some(inner) = NonNull::new(service) {
            Ok(MeetingService { inner, sdk })
        } else {
            Err(Error::new_rust(
                "ZOOMSDK_CreateMeetingService returned null",
            ))
        }
    }

    /// Join meeting with web uri.
    pub fn handle_zoom_web_uri_protocol_action(&mut self, uri: &str) -> ZoomResult<()> {
        let uri = str_to_u16_vec(uri);
        let p = self.inner.as_ptr();
        unsafe {
            ffi::ZOOMSDK_IMeetingsService_HandleZoomWebUriProtocolAction(p, uri.as_ptr())
                .err_wrap(true)
        }
    }
}
