use gtk::{gio, prelude::ApplicationExt, glib};
use nixos_conf_editor::{config::APP_ID, ui::window::AppModel};
use relm4::{RelmApp, actions::AccelsPlus};

relm4::new_action_group!(WindowActionGroup, "window");
relm4::new_stateless_action!(SearchAction, WindowActionGroup, "search");

fn main() {
    gtk::init().unwrap();
    pretty_env_logger::init();
	glib::set_application_name("Configuration Editor");
    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::empty());
    app.set_resource_base_path(Some("/dev/vlinkz/NixosConfEditor"));
    app.set_accelerators_for_action::<SearchAction>(&["<Control>f"]);
    let app = RelmApp::with_app(app);
    app.run::<AppModel>(());
}
