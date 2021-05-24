use native_windows_gui as nwg;
use std::cell::RefCell;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use winapi::um::libloaderapi::GetModuleHandleA;

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let mut window = Default::default();
    let mut log_label = Default::default();
    let layout = Default::default();

    nwg::Window::builder()
        .size((300, 115))
        // .position((300, 300))
        .title("Rust Zoom SDK")
        .build(&mut window)
        .unwrap();

    nwg::Label::builder()
        .parent(&window)
        .text("Starting...")
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

    // TODO: unfortunately, full_bind_event_handler requires static lifetime
    let zoom_state = RefCell::new(ZoomState::default());

    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, handle| {
        use nwg::Event as E;

        match evt {
            E::OnWindowClose => {
                if &handle == &events_window as &nwg::Window {
                    nwg::stop_thread_dispatch();
                }
            }
            E::OnInit => {
                println!("OnInit");
                catch_error(|| join_meeting(&zoom_state));
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

#[derive(Debug, Default)]
struct ZoomState<'a> {
    meeting: Option<zoom_sdk::meeting::MeetingService<'a>>,
    auth: Option<Pin<Box<zoom_sdk::auth::AuthService<'a>>>>,
}

fn join_meeting<'a, 'b>(
    state_cell: &'a RefCell<ZoomState<'b>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let init_param = zoom_sdk::InitParam::new()
        .branding_name(Some("RustWrapper")) // working
        .res_instance(unsafe { GetModuleHandleA(ptr::null()) })
        .ui_window_icon_big_id(2734) // working
        .ui_window_icon_small_id(2734)
        .em_language_id(zoom_sdk::SdkLanguageId::English) // working
        .enable_log_by_default(true)
        .enable_generate_dump(true);
    zoom_sdk::init_sdk(&init_param).expect("Initialization failed");
    println!("Initialized");
    let mut state = state_cell.borrow_mut();
    state.meeting = Some(zoom_sdk::create_meeting_service()?);
    state.auth = Some(zoom_sdk::create_auth_service()?);
    let auth = state.auth.as_mut().unwrap();
    auth.set_event(zoom_sdk::auth::AuthServiceEvent {
        authentication_return: Box::new(|auth, res| {
            catch_error(|| {
                println!("AuthResult {:?}", res);
                let username = std::env::var("ZOOM_LOGIN_USER")?;
                let password = std::env::var("ZOOM_LOGIN_PASS")?;
                auth.login(&username, &password, false)?;
                Ok(())
            });
        }),
        login_return: Box::new(|_auth, status| {
            catch_error(|| {
                println!("login status {:?}", status);
                if let zoom_sdk::auth::LoginStatus::Success(info) = status {
                    let name = info.get_display_name();
                    println!("name {}", name);
                    let uri = std::env::var("ZOOM_URI")?;
                    // TODO: meeting
                    // RefCell::borrow_mut(&state_cell)
                    //     .meeting
                    //     .as_ref()
                    //     .unwrap()
                    //     .handle_zoom_web_uri_protocol_action(&uri)?;
                }
                Ok(())
            });
        }),
    })?;
    auth.sdk_auth()?;
    println!("auth service created");
    Ok(())
}

fn catch_error(f: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>) {
    f().unwrap_or_else(|e| {
        eprintln!("{0}, detail {0:?}", &e);
        // self.report_error(&format!("{}", &e));
    });
}

// fn init_status(&self, text: &str) {
//     self.init_status_label.set_text(text);
// }
//
// fn report_error(&self, err_message: &str) {
//     nwg::modal_error_message(&self.window, "Error", err_message);
// }
//
