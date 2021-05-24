use crate::{ffi, str_to_u16_vec, Error, ErrorExt, ZoomResult};
use std::marker::PhantomPinned;
use std::panic::catch_unwind;
use std::pin::Pin;
use std::ptr::NonNull;
use std::{fmt, mem, ptr};

/// Meeting Service
pub struct MeetingService<'a> {
    /// This struct is not supposed to be Send nor Sync
    inner: NonNull<ffi::ZOOMSDK_IMeetingService>,
    event_data: Option<EventObject<'a>>,
    _marker: PhantomPinned,
}

/// C++ sees this as class that inherits from IMeetingServiceEvent
#[repr(C)]
pub struct EventObject<'a> {
    base: ffi::ZoomGlue_MeetingServiceEvent,
    service: NonNull<MeetingService<'a>>,
    events: Box<dyn MeetingServiceEvent + 'a>,
}

pub trait MeetingServiceEvent {
    fn meeting_status_changed(&self, _meeting: &MeetingService, _status: MeetingStatus) {}
    fn meeting_statistics_warning_notification(
        &self,
        _meeting: &MeetingService,
        _typ: StatisticsWarningType,
    ) {
    }
}

impl fmt::Debug for EventObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("zoom_sdk::EventObject").finish()
    }
}

#[derive(Debug)]
pub enum StatisticsWarningType {}

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
    pub(crate) fn new() -> ZoomResult<Self> {
        let mut service = ptr::null_mut();
        unsafe { ffi::ZOOMSDK_CreateMeetingService(&mut service) }.err_wrap(true)?;
        if let Some(inner) = NonNull::new(service) {
            Ok(MeetingService {
                inner,
                event_data: None,
                _marker: Default::default(),
            })
        } else {
            Err(Error::new_rust(
                "ZOOMSDK_CreateMeetingService returned null",
            ))
        }
    }

    /// Join meeting with web uri.
    pub fn handle_zoom_web_uri_protocol_action(&self, uri: &str) -> ZoomResult<()> {
        let uri = str_to_u16_vec(uri);
        let p = self.inner.as_ptr();
        unsafe {
            ffi::ZoomGlue_IMeetingService_HandleZoomWebUriProtocolAction(p, uri.as_ptr())
                .err_wrap(true)
        }
    }

    pub fn set_event(
        self: &mut Pin<Box<Self>>,
        events: Box<dyn MeetingServiceEvent + 'a>,
    ) -> ZoomResult<()> {
        // Pinned because the self-referencing struct and a pointer passed to C++.
        unsafe {
            let service = Pin::get_unchecked_mut(self.as_mut());
            let service_p = NonNull::from(service as &MeetingService);
            let data = EventObject {
                base: mem::zeroed(),
                service: service_p,
                events,
            };
            service.event_data = Some(data);
            let object_base = &mut service.event_data.as_mut().unwrap().base;
            ffi::ZoomGlue_MeetingServiceEvent_PlacementNew(object_base);
            object_base.cbMeetingStatusChanged = Some(on_meeting_status_changed);
            object_base.cbMeetingStatisticsWarningNotification =
                Some(on_meeting_statistics_warning_notification);
            // safe cast because of inheritance
            let interface_p = object_base as *mut ffi::ZoomGlue_MeetingServiceEvent
                as *mut ffi::ZOOMSDK_IMeetingServiceEvent;
            ffi::ZoomGlue_IMeetingService_SetEvent(service.inner.as_ptr(), interface_p)
                .err_wrap(true)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum MeetingStatus {
    /// No meeting is running.
    Idle,
    /// Connect to the meeting server status.
    Connecting,
    /// Waiting for the host to start the meeting.
    WaitingForHost,
    /// Meeting is ready, in meeting status.
    InMeeting,
    /// Disconnect the meeting server, leave meeting status.
    Disconnecting,
    /// Reconnecting meeting server status.
    Reconnecting,
    /// Failed to connect the meeting server.
    Failed(MeetingFailCode),
    /// Meeting ends.
    Ended(MeetingEndReason),
    /// Unknown status.
    Unknown,
    /// Meeting is locked to prevent the further participants to join the meeting.
    Locked,
    /// Meeting is open and participants can join the meeting.
    Unlocked,
    /// Participants who join the meeting before the start are in the waiting room.
    InWaitingRoom,
    /// Upgrade the attendees to panelist in webinar.
    WebinarPromote,
    /// Downgrade the attendees from the panelist.
    WebinarDepromote,
    /// Join the breakout room.
    JoinBreakoutRoom,
    /// Leave the breakout room.
    LeaveBreakoutRoom,
    /// Waiting for the additional secret key.
    WaitingExternalSessionKey,
    /// Unmapped.
    Unmapped(i32),
}

fn map_meeting_status(status: ffi::ZOOMSDK_MeetingStatus, i_result: i32) -> MeetingStatus {
    match status {
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_IDLE => MeetingStatus::Idle,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_CONNECTING => MeetingStatus::Connecting,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_WAITINGFORHOST => MeetingStatus::WaitingForHost,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_INMEETING => MeetingStatus::InMeeting,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_DISCONNECTING => MeetingStatus::Disconnecting,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_RECONNECTING => MeetingStatus::Reconnecting,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_FAILED => {
            MeetingStatus::Failed(map_fail(i_result))
        }
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_ENDED => MeetingStatus::Ended(map_end(i_result)),
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_UNKNOW => MeetingStatus::Unknown,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_LOCKED => MeetingStatus::Locked,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_UNLOCKED => MeetingStatus::Unlocked,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_IN_WAITING_ROOM => MeetingStatus::InWaitingRoom,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_WEBINAR_PROMOTE => MeetingStatus::WebinarPromote,
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_WEBINAR_DEPROMOTE => {
            MeetingStatus::WebinarDepromote
        }
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_JOIN_BREAKOUT_ROOM => {
            MeetingStatus::JoinBreakoutRoom
        }
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_LEAVE_BREAKOUT_ROOM => {
            MeetingStatus::LeaveBreakoutRoom
        }
        ffi::ZOOMSDK_MeetingStatus_MEETING_STATUS_WAITING_EXTERNAL_SESSION_KEY => {
            MeetingStatus::WaitingExternalSessionKey
        }
        _ => MeetingStatus::Unmapped(status),
    }
}

/// Meeting failure code.
#[derive(Debug)]
enum MeetingFailCode {
    /// Start meeting successfully.
    MeetingSuccess,
    /// Network error.
    MeetingFailNetworkErr,
    /// Reconnect error.
    MeetingFailReconnectErr,
    /// Multi-media Router error.
    MeetingFailMmrErr,
    /// Password is wrong.
    MeetingFailPasswordErr,
    /// Session error.
    MeetingFailSessionErr,
    /// Meeting is over.
    MeetingFailMeetingOver,
    /// Meeting has not begun.
    MeetingFailMeetingNotStart,
    /// Meeting does not exist.
    MeetingFailMeetingNotExist,
    /// The capacity of meeting is full.
    MeetingFailMeetingUserFull,
    /// The client is incompatible.
    MeetingFailClientIncompatible,
    /// The Multi-media router is not founded.
    MeetingFailNoMmr,
    /// The meeting is locked.
    MeetingFailConfLocked,
    /// The meeting is failed because of the restriction by the same account.
    MeetingFailMeetingRestricted,
    /// The meeting is restricted by the same account while the attendee is allowed to join before the host.
    MeetingFailMeetingRestrictedJbh,
    /// Unable to send web request.
    MeetingFailCannotEmitWebRequest,
    ///The token is expired.
    MeetingFailCannotStartTokenExpire,
    /// Video hardware or software error.
    SessionVideoErr,
    /// Audio autostart error.
    SessionAudioAutoStartErr,
    /// The number of webinar registered has reached the upper limit.
    MeetingFailRegisterWebinarFull,
    /// Register webinar with the role of webinar host.
    MeetingFailRegisterWebinarHostRegister,
    /// Register webinar with the role of panelist member.
    MeetingFailRegisterWebinarPanelistRegister,
    /// Register webinar with the denied email.
    MeetingFailRegisterWebinarDeniedEmail,
    /// Webinar request to login.
    MeetingFailEnforceLogin,
    /// Invalid for Windows SDK.
    ConfFailZcCertificateChanged,
    /// Vanity conference ID does not exist.
    ConfFailVanityNotExist,
    /// Join webinar with the same email.
    ConfFailJoinWebinarWithSameEmail,
    /// Meeting settings is not allowed to start a meeting.
    ConfFailDisallowHostMeeting,
    /// Disabled to write the configure file.
    MeetingFailWriteConfigFile,
    /// Forbidden to join the internal meeting.
    MeetingFailForbidToJoinInternalMeeting,
    /// Removed by the host.
    ConfFailRemovedByHost,
    /// Unmapped.
    Unmapped(i32),
}

fn map_fail(i_result: i32) -> MeetingFailCode {
    match i_result {
        ffi::ZOOMSDK_MeetingFailCode_MEETING_SUCCESS => MeetingFailCode::MeetingSuccess,
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_NETWORK_ERR => {
            MeetingFailCode::MeetingFailNetworkErr
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_RECONNECT_ERR => {
            MeetingFailCode::MeetingFailReconnectErr
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MMR_ERR => MeetingFailCode::MeetingFailMmrErr,
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_PASSWORD_ERR => {
            MeetingFailCode::MeetingFailPasswordErr
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_SESSION_ERR => {
            MeetingFailCode::MeetingFailSessionErr
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_OVER => {
            MeetingFailCode::MeetingFailMeetingOver
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_START => {
            MeetingFailCode::MeetingFailMeetingNotStart
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_NOT_EXIST => {
            MeetingFailCode::MeetingFailMeetingNotExist
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_USER_FULL => {
            MeetingFailCode::MeetingFailMeetingUserFull
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_CLIENT_INCOMPATIBLE => {
            MeetingFailCode::MeetingFailClientIncompatible
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_NO_MMR => MeetingFailCode::MeetingFailNoMmr,
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_CONFLOCKED => {
            MeetingFailCode::MeetingFailConfLocked
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED => {
            MeetingFailCode::MeetingFailMeetingRestricted
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_MEETING_RESTRICTED_JBH => {
            MeetingFailCode::MeetingFailMeetingRestrictedJbh
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_EMIT_WEBREQUEST => {
            MeetingFailCode::MeetingFailCannotEmitWebRequest
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_CANNOT_START_TOKENEXPIRE => {
            MeetingFailCode::MeetingFailCannotStartTokenExpire
        }
        ffi::ZOOMSDK_MeetingFailCode_SESSION_VIDEO_ERR => MeetingFailCode::SessionVideoErr,
        ffi::ZOOMSDK_MeetingFailCode_SESSION_AUDIO_AUTOSTARTERR => {
            MeetingFailCode::SessionAudioAutoStartErr
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_FULL => {
            MeetingFailCode::MeetingFailRegisterWebinarFull
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_HOSTREGISTER => {
            MeetingFailCode::MeetingFailRegisterWebinarHostRegister
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_PANELISTREGISTER => {
            MeetingFailCode::MeetingFailRegisterWebinarPanelistRegister
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_REGISTERWEBINAR_DENIED_EMAIL => {
            MeetingFailCode::MeetingFailRegisterWebinarDeniedEmail
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_ENFORCE_LOGIN => {
            MeetingFailCode::MeetingFailEnforceLogin
        }
        ffi::ZOOMSDK_MeetingFailCode_CONF_FAIL_ZC_CERTIFICATE_CHANGED => {
            MeetingFailCode::ConfFailZcCertificateChanged
        }
        ffi::ZOOMSDK_MeetingFailCode_CONF_FAIL_VANITY_NOT_EXIST => {
            MeetingFailCode::ConfFailVanityNotExist
        }
        ffi::ZOOMSDK_MeetingFailCode_CONF_FAIL_JOIN_WEBINAR_WITHSAMEEMAIL => {
            MeetingFailCode::ConfFailJoinWebinarWithSameEmail
        }
        ffi::ZOOMSDK_MeetingFailCode_CONF_FAIL_DISALLOW_HOST_MEETING => {
            MeetingFailCode::ConfFailDisallowHostMeeting
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_WRITE_CONFIG_FILE => {
            MeetingFailCode::MeetingFailWriteConfigFile
        }
        ffi::ZOOMSDK_MeetingFailCode_MEETING_FAIL_FORBID_TO_JOIN_INTERNAL_MEETING => {
            MeetingFailCode::MeetingFailForbidToJoinInternalMeeting
        }
        ffi::ZOOMSDK_MeetingFailCode_CONF_FAIL_REMOVED_BY_HOST => {
            MeetingFailCode::ConfFailRemovedByHost
        }
        _ => MeetingFailCode::Unmapped,
    }
}

/// Meeting failure code.
#[derive(Debug)]
enum MeetingEndReason {
    /// For initialization.
    None,
    /// Kicked by host.
    KickByHost,
    /// Ended by host.
    EndByHost,
    /// JBH times out.
    JBHTimeOut,
    /// No attendee.
    NoAttendee,
    /// Host starts another meeting.
    HostStartAnotherMeeting,
    /// Free meeting times out.
    FreeMeetingTimeOut,
    /// Network is broken.
    NetworkBroken,
    /// Unmapped.
    Unmapped(i32),
}

fn map_end(i_result: i32) -> MeetingEndReason {
    match i_result {
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_None => MeetingEndReason::None,
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_KickByHost => MeetingEndReason::KickByHost,
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_EndByHost => MeetingEndReason::EndByHost,
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_JBHTimeOut => MeetingEndReason::JBHTimeOut,
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_NoAttendee => MeetingEndReason::NoAttendee,
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_HostStartAnotherMeeting => {
            MeetingEndReason::HostStartAnotherMeeting
        }
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_FreeMeetingTimeOut => {
            MeetingEndReason::FreeMeetingTimeOut
        }
        ffi::ZOOMSDK_MeetingEndReason_EndMeetingReason_NetworkBroken => {
            MeetingEndReason::NetworkBroken
        }
        _ => MeetingFailCode::Unmapped,
    }
}

unsafe extern "C" fn on_meeting_status_changed(
    this: *mut ffi::ZOOMSDK_IMeetingServiceEvent,
    status: ffi::ZOOMSDK_MeetingStatus,
    i_result: i32,
) {
    let _ = catch_unwind(|| {
        events_callback(this, |events, service| {
            events.meeting_status_changed(service, map_meeting_status(status, i_result));
        });
    });
}

unsafe extern "C" fn on_meeting_statistics_warning_notification(
    this: *mut ffi::ZOOMSDK_IMeetingServiceEvent,
    typ: ffi::ZOOMSDK_StatisticsWarningType,
) {
    let _ = catch_unwind(|| {
        events_callback(this, |events, service| {
            events.meeting_statistics_warning_notification(service, todo!());
        });
    });
}

unsafe fn events_callback(
    this: *mut ffi::ZOOMSDK_IMeetingServiceEvent,
    mut f: impl FnMut(&mut Box<dyn MeetingServiceEvent>, &mut MeetingService),
) {
    let service = (*(this as *mut EventObject)).service.as_mut();
    let mut tmp_data = None;
    // callback may not call set_event, as that would mutate the running closure
    // so temporary swap event data.
    mem::swap(&mut service.event_data, &mut tmp_data);
    let events = &mut tmp_data.as_mut().unwrap().events;
    f(events, service);
    mem::swap(&mut service.event_data, &mut tmp_data);
}
