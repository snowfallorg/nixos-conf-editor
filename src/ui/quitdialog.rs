use adw::prelude::*;
use relm4::*;
use super::window::{AppModel, AppMsg};

pub struct QuitCheckModel {
    hidden: bool,
    busy: bool,
}

pub enum QuitCheckMsg {
    Show,
    Save,
    Rebuild,
    Quit,
}

impl Model for QuitCheckModel {
    type Msg = QuitCheckMsg;
    type Widgets = QuitCheckWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for QuitCheckModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        QuitCheckModel {
            hidden: true,
            busy: false,
        }
    }

    fn update(
        &mut self,
        msg: QuitCheckMsg,
        _components: &(),
        _sender: Sender<QuitCheckMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            QuitCheckMsg::Show => {
                self.hidden = false;
                self.busy = false;
            },
            QuitCheckMsg::Save => {
                self.busy = true;
                send!(parent_sender, AppMsg::SaveQuit);
            },
            QuitCheckMsg::Rebuild => {
                self.hidden = true;
                send!(parent_sender, AppMsg::Rebuild);
            },
            QuitCheckMsg::Quit => {
                relm4::gtk_application().quit();
            },
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<QuitCheckModel, AppModel> for QuitCheckWidgets {
    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_resizable: false,
            set_sensitive: watch!(!model.busy),
            set_text: Some("Save Changes?"),
            set_secondary_text: Some("Unsaved changes will be lost. You should rebuild your system now to ensure you configured everything properly. You can also save your configuration, however is is possible that your configuration is save in an unbuildable state."),
            set_default_width: 500,
            add_button: args!("Quit", gtk::ResponseType::Close),
            add_button: args!("Save", gtk::ResponseType::Reject),
            add_button: args!("Rebuild", gtk::ResponseType::Accept),
            connect_response(sender) => move |_, resp| {
                send!(sender, match resp {
                    gtk::ResponseType::Accept => QuitCheckMsg::Rebuild,
                    gtk::ResponseType::Reject => QuitCheckMsg::Save,
                    gtk::ResponseType::Close => QuitCheckMsg::Quit,
                    _ => unreachable!(),
                });
            },
        }
    }

    fn post_init() {
        let rebuild_widget = dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        rebuild_widget.add_css_class("suggested-action");
        let save_widget = dialog
            .widget_for_response(gtk::ResponseType::Reject)
            .expect("No button for reject response set");
        save_widget.add_css_class("warning");
        let quit_widget = dialog
            .widget_for_response(gtk::ResponseType::Close)
            .expect("No button for close response set");
        quit_widget.add_css_class("destructive-action");
    }
}
