use super::window::AppMsg;
use adw::prelude::*;
use relm4::*;

pub struct QuitCheckModel {
    hidden: bool,
    busy: bool,
}

#[derive(Debug)]
pub enum QuitCheckMsg {
    Show,
    Save,
    Rebuild,
    Quit,
}

#[relm4::component(pub)]
impl SimpleComponent for QuitCheckModel {
    type Init = gtk::Window;
    type Input = QuitCheckMsg;
    type Output = AppMsg;
    type Widgets = QuitCheckWidgets;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&init),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_resizable: false,
            #[watch]
            set_sensitive: !model.busy,
            set_text: Some("Save Changes?"),
            set_secondary_text: Some("Unsaved changes will be lost. You should rebuild your system now to ensure you configured everything properly. You can also save your configuration, however is is possible that your configuration is save in an unbuildable state."),
            set_default_width: 500,
            add_button: ("Quit", gtk::ResponseType::Close),
            add_button: ("Save", gtk::ResponseType::Reject),
            add_button: ("Rebuild", gtk::ResponseType::Accept),
            connect_response[sender] => move |_, resp| {
                sender.input(match resp {
                    gtk::ResponseType::Accept => QuitCheckMsg::Rebuild,
                    gtk::ResponseType::Reject => QuitCheckMsg::Save,
                    gtk::ResponseType::Close => QuitCheckMsg::Quit,
                    _ => unreachable!(),
                });
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = QuitCheckModel {
            hidden: true,
            busy: false,
        };

        let widgets = view_output!();

        let rebuild_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        rebuild_widget.add_css_class("suggested-action");
        let save_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::Reject)
            .expect("No button for reject response set");
        save_widget.add_css_class("warning");
        let quit_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::Close)
            .expect("No button for close response set");
        quit_widget.add_css_class("destructive-action");

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            QuitCheckMsg::Show => {
                self.hidden = false;
                self.busy = false;
            }
            QuitCheckMsg::Save => {
                self.busy = true;
                sender.output(AppMsg::SaveQuit);
            }
            QuitCheckMsg::Rebuild => {
                self.hidden = true;
                sender.output(AppMsg::Rebuild);
            }
            QuitCheckMsg::Quit => {
                relm4::main_application().quit();
            }
        }
    }
}
