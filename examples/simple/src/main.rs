use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::cell::{Ref, RefCell};
use std::ptr;
use winapi::um::libloaderapi::GetModuleHandleA;

#[derive(NwgUi, Default)]
pub struct BasicApp {
    #[nwg_control(size: (300, 200), position: (300, 300), title: "Basic example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnInit: [BasicApp::init], OnWindowClose: [BasicApp::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Heisenberg", focus: true)]
    #[nwg_layout_item(layout: grid, row: 0, col: 0)]
    name_edit: nwg::TextInput,

    #[nwg_control(text: "Say my name")]
    #[nwg_layout_item(layout: grid, col: 0, row: 1, row_span: 2)]
    #[nwg_events( OnButtonClick: [BasicApp::say_hello] )]
    hello_button: nwg::Button,

    #[nwg_control(text: "Initializing...")]
    #[nwg_layout_item(layout: grid, col: 0, row: 3, row_span: 2)]
    init_status_label: nwg::Label,

    zoom_state: RefCell<Option<ZoomState>>,
}

impl BasicApp {
    fn init(&self) {
        self.catch_error(|| join_meeting(self));
    }

    fn say_hello(&self) {
        nwg::modal_info_message(
            &self.window,
            "Hello",
            &format!("Hello {}", self.name_edit.text()),
        );
    }

    fn init_status(&self, text: &str) {
        self.init_status_label.set_text(text);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn report_error(&self, err_message: &str) {
        nwg::modal_error_message(&self.window, "Error", err_message);
    }

    fn catch_error(&self, f: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>) {
        f().unwrap_or_else(|e| {
            println!("{0}, detail {0:?}", &e);
            self.report_error(&format!("{}", &e));
        });
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}

struct ZoomState {
    sdk: zoom_sdk::Sdk,
}

fn join_meeting(app: &BasicApp) -> Result<(), Box<dyn std::error::Error>> {
    let sdk = zoom_sdk::InitParam::new()
        .branding_name(Some("RustWrapper")) // working
        .res_instance(unsafe { GetModuleHandleA(ptr::null()) })
        .ui_window_icon_big_id(2734) // working
        .ui_window_icon_small_id(2734)
        .em_language_id(zoom_sdk::SdkLanguageId::English) // working
        .enable_log_by_default(true)
        .enable_generate_dump(true)
        .init_sdk()?;
    println!("Initialized");
    // *app.zoom_state.borrow_mut() = Some(ZoomState { sdk });
    // let sdk = Ref::map(app.zoom_state.borrow(), |s| &s.as_ref().unwrap().sdk);
    let mut state = app.zoom_state.borrow_mut();
    *state = Some(ZoomState { sdk });
    let sdk = &state.as_ref().unwrap().sdk;
    let meeting = sdk.create_meeting_service()?;
    let mut auth = sdk.create_auth_service()?;
    auth.as_mut().set_event(zoom_sdk::auth::AuthServiceEvent {
        authentication_return: Box::new(|auth, res| {
            app.catch_error(|| {
                app.init_status(&format!("Authentication {:?}", res));
                println!("AuthResult {:?}", res);
                let username = std::env::var("ZOOM_LOGIN_USER")?;
                let password = std::env::var("ZOOM_LOGIN_PASS")?;
                auth.login(&username, &password, false)?;
                Ok(())
            });
        }),
        login_return: Box::new(|_auth, status| {
            app.catch_error(|| {
                println!("login status {:?}", status);
                if let zoom_sdk::auth::LoginStatus::Success(info) = status {
                    let name = info.get_display_name();
                    println!("name {}", name);
                    app.init_status(&format!("Logged in as {}", name));
                    let uri = std::env::var("ZOOM_URI")?;
                    meeting.handle_zoom_web_uri_protocol_action(&uri)?;
                }
                Ok(())
            });
        }),
    })?;
    auth.sdk_auth()?;
    println!("auth service created");
    Ok(())
}
