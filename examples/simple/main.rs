use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use nwg::NativeUi;
use std::ptr;
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

fn main() -> Result<(), zoom_sdk::error::Error> {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    let mut zoom = zoom_sdk::InitParam::new()
        .branding_name(Some("MyBranding"))
        .res_instance(unsafe { GetModuleHandleA(ptr::null()) })
        .ui_window_icon_big_id(2734)
        .ui_window_icon_small_id(2734)
        .em_language_id(zoom_sdk::SdkLanguageId::German)
        .enable_log_by_default(true)
        .enable_generate_dump(true)
        .init_sdk()?;
    println!("Initialized");
    let mut auth = zoom.create_auth_service()?;
    auth.set_event(zoom_sdk::auth::AuthServiceEvent {
        authentication_return: Box::new(|res| {
            app.init_status(&format!("Authentication {:?}", res))
        }),
        login_return: Box::new(|status| {
            println!("login status {:?}", status);
            if let zoom_sdk::auth::LoginStatus::Success(info) = status {
                let name = info.get_display_name();
                println!("name {}", name);
                app.init_status(&format!("Logged in as {}", name));
            }
        }),
    })?;
    auth.sdk_auth()?;
    println!("auth service created");
    nwg::dispatch_thread_events();
    Ok(())
}
