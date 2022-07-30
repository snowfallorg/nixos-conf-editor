use relm4::*;
use adw::prelude::*;
use crate::config;

use super::window::AppMsg;

pub struct AboutModel {
    hidden: bool,
}

#[derive(Debug)]
pub enum AboutMsg {
    Show,
    Hide,
}

#[relm4::component(pub)]
impl SimpleComponent for AboutModel {
    type InitParams = gtk::Window;
    type Input = AboutMsg;
    type Output = AppMsg;
    type Widgets = AboutWidgets;

    view! {
        dialog = gtk::AboutDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_authors: &["<a href=\"https://github.com/vlinkz\">Victor Fuentes</a>"],
            set_copyright: Some("© 2022 Victor Fuentes"),
            set_license_type: gtk::License::MitX11,
            set_program_name: Some("NixOS Configuration Editor"),
            set_version: Some(config::VERSION),
            set_logo_icon_name: Some(config::APP_ID),
            set_sensitive: true,
            connect_close_request[sender] => move |_| {
                sender.input(AboutMsg::Hide);
                gtk::Inhibit(true)
            }
        }
    }

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AboutModel {
            hidden: true,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
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
