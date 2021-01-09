use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::ptr;
use std::sync::RwLock;
use winapi::um::libloaderapi::GetModuleHandleA;

#[derive(Default, NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (300, 200), position: (300, 300), title: "Basic example", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [BasicApp::say_goodbye] )]
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
}

impl BasicApp {
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

    fn say_goodbye(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    let zoom = zoom_sdk::InitParam::new()
        .branding_name(Some("MyBranding"))
        .res_instance(unsafe { GetModuleHandleA(ptr::null()) })
        .ui_window_icon_big_id(2734)
        .ui_window_icon_small_id(2734)
        .em_language_id(zoom_sdk::SdkLanguageId::German)
        .enable_log_by_default(true)
        .enable_generate_dump(true)
        .init_sdk()?;
    println!("Initialized");
    let mut meeting = zoom.create_meeting_service()?;
    let auth = zoom.create_auth_service()?;
    auth.set_event(zoom_sdk::auth::AuthServiceEvent {
        authentication_return: Box::new(|auth, res| {
            app.init_status(&format!("Authentication {:?}", res));
            println!("AuthResult {:?}", res);
            let username = std::env::var("ZOOM_LOGIN_USER").unwrap();
            let password = std::env::var("ZOOM_LOGIN_PASS").unwrap();
            auth.login(&username, &password, false);
        }),
        login_return: Box::new(|auth, status| {
            println!("login status {:?}", status);
            if let zoom_sdk::auth::LoginStatus::Success(info) = status {
                let name = info.get_display_name();
                println!("name {}", name);
                app.init_status(&format!("Logged in as {}", name));
                meeting.handle_zoom_web_uri_protocol_action("");
            }
        }),
    })?;
    auth.sdk_auth()?;
    println!("auth service created");
    nwg::dispatch_thread_events();
    Ok(())
}
