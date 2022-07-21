use adw::prelude::*;
use relm4::{Model, ComponentUpdate, Sender, Widgets, send};
use crate::config;

use super::window::{AppModel, AppMsg};

pub struct AboutModel {
    hidden: bool,
}

pub enum AboutMsg {
    Show,
    Hide,
}

impl Model for AboutModel {
    type Msg = AboutMsg;
    type Widgets = AboutWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for AboutModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        AboutModel {
            hidden: true,
        }
    }

    fn update(
        &mut self,
        msg: AboutMsg,
        _components: &(),
        _sender: Sender<AboutMsg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            AboutMsg::Show => {
                self.hidden = false;
            },
            AboutMsg::Hide => {
                self.hidden = true;
            }
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<AboutModel, AppModel> for AboutWidgets {
    view! {
        dialog = gtk::AboutDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_authors: &["<a href=\"https://github.com/vlinkz\">Victor Fuentes</a>"],
            set_copyright: Some("© 2022 Victor Fuentes"),
            set_license_type: gtk::License::MitX11,
            set_program_name: Some("NixOS Configuration Editor"),
            set_version: Some(config::VERSION),
            set_logo_icon_name: Some(config::APP_ID),
            set_sensitive: true,
            connect_close_request => move |_| {
                send!(sender, AboutMsg::Hide);
                gtk::Inhibit(true)
            }
        }
    }
}
