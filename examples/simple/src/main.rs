use native_windows_gui as nwg;
use std::cell::RefCell;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use winapi::um::libloaderapi::GetModuleHandleA;
use zoom_sdk_windows::auth::{AuthResult, AuthService, AuthServiceEvent, LoginStatus};
use zoom_sdk_windows::meeting::{
    MeetingService, MeetingServiceEvent, MeetingStatus, StatisticsWarningType,
};

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let mut window = Default::default();
    let mut log_label = Default::default();
    let layout = Default::default();

    nwg::Window::builder()
        .size((300, 115))
        .title("Rust Zoom SDK")
        .build(&mut window)
        .unwrap();

    nwg::Label::builder()
        .parent(&window)
        .text("Example")
        .build(&mut log_label)
        .unwrap();

    nwg::GridLayout::builder()
        .parent(&window)
        .spacing(1)
        .child_item(nwg::GridLayoutItem::new(&log_label, 0, 1, 1, 2))
        .build(&layout)
        .unwrap();

    let window = Rc::new(window);
    let events_window = window.clone();

    // unfortunately, full_bind_event_handler requires static lifetime
    let zoom_state = Rc::new(RefCell::new(ZoomState {
        window: window.clone(),
        services: None,
    }));

    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, handle| {
        use nwg::Event as E;

        match evt {
            E::OnWindowClose => {
                if &handle == &events_window as &nwg::Window {
                    nwg::stop_thread_dispatch();
                }
            }
            E::OnInit => {
                catch_error(|| join_meeting(&zoom_state));
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

fn join_meeting(state: &Rc<RefCell<ZoomState>>) -> Result<(), Box<dyn std::error::Error>> {
    let init_param = zoom_sdk_windows::InitParam::new()
        .branding_name(Some("RustWrapper")) // working
        .res_instance(unsafe { GetModuleHandleA(ptr::null()) })
        .ui_window_icon_big_id(2734) // working
        .ui_window_icon_small_id(2734)
        .em_language_id(zoom_sdk_windows::SdkLanguageId::English) // working
        .enable_log_by_default(true)
        .enable_generate_dump(true);
    zoom_sdk_windows::init_sdk(&init_param).expect("Initialization failed");
    println!("Zoom initialized");
    let mut state_borrow = state.borrow_mut();
    state_borrow.services = Some(ZoomServices {
        meeting: zoom_sdk_windows::create_meeting_service()?,
        auth: zoom_sdk_windows::create_auth_service()?,
    });
    let meeting = &mut state_borrow.services.as_mut().unwrap().meeting;
    meeting.set_event(Box::new(EventImpl {
        state: state.clone(),
    }))?;
    let auth = &mut state_borrow.services.as_mut().unwrap().auth;
    // TODO: reuse the same EventImpl box for meeting
    auth.set_event(Box::new(EventImpl {
        state: state.clone(),
    }))?;
    println!("Zoom services created");
    auth.sdk_auth()?;
    Ok(())
}

#[derive(Default)]
struct ZoomState {
    window: Rc<nwg::Window>,
    services: Option<ZoomServices>,
}

#[derive(Debug)]
struct ZoomServices {
    meeting: Pin<Box<MeetingService<'static>>>,
    auth: Pin<Box<AuthService<'static>>>,
}

struct EventImpl {
    state: Rc<RefCell<ZoomState>>,
}

impl AuthServiceEvent for EventImpl {
    fn authentication_return(&self, auth: &AuthService, auth_result: AuthResult) {
        catch_error(|| {
            println!("AuthResult {:?}", auth_result);
            let username = std::env::var("ZOOM_LOGIN_USER")?;
            let password = std::env::var("ZOOM_LOGIN_PASS")?;
            auth.login(&username, &password, false)?;
            Ok(())
        });
    }

    fn login_return(&self, _auth: &AuthService, login_status: LoginStatus) {
        catch_error(|| {
            println!("LoginStatus {:?}", login_status);
            if let zoom_sdk_windows::auth::LoginStatus::Success(info) = login_status {
                let name = info.get_display_name();
                println!("Logged with name {}", name);
                let uri = std::env::var("ZOOM_URI")?;
                let state = RefCell::borrow_mut(&self.state);
                let meeting = &state.services.as_ref().unwrap().meeting;
                meeting.handle_zoom_web_uri_protocol_action(&uri)?;
            }
            Ok(())
        });
    }
}

impl MeetingServiceEvent for EventImpl {
    fn meeting_status_changed(&self, _meeting: &MeetingService, status: MeetingStatus) {
        println!("Meeting status {:?}", &status);
        if let MeetingStatus::Ended(..) = status {
            RefCell::borrow(&self.state).window.close();
        }
    }

    fn meeting_statistics_warning_notification(
        &self,
        _meeting: &MeetingService,
        typ: StatisticsWarningType,
    ) {
        println!("Meeting statistics warning {:?}", typ);
    }
}

fn catch_error(f: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>) {
    f().unwrap_or_else(|e| {
        eprintln!("{0}, detail {0:?}", &e);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {
        assert_eq!(2, 1 + 1);
    }
}
