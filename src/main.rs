use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib, prelude::ApplicationExt};
use nixos_conf_editor::{
    config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR},
    ui::window::AppModel,
};
use relm4::{actions::AccelsPlus, RelmApp};

relm4::new_action_group!(WindowActionGroup, "window");
relm4::new_stateless_action!(SearchAction, WindowActionGroup, "search");

fn main() {
    gtk::init().unwrap();
    pretty_env_logger::init();
    setup_gettext();
    glib::set_application_name(&gettext("Configuration Editor"));
    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::empty());
    app.set_resource_base_path(Some("/dev/vlinkz/NixosConfEditor"));
    app.set_accelerators_for_action::<SearchAction>(&["<Control>f"]);
    let app = RelmApp::with_app(app);
    app.run::<AppModel>(());
}

fn setup_gettext() {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to bind the text domain codeset to UTF-8");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
}
