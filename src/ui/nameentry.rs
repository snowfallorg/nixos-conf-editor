use super::window::*;
use adw::prelude::*;
use relm4::*;

#[tracker::track]
pub struct NameEntryModel {
    hidden: bool,
    msg: String,
    existing: Vec<String>,
    text: String,
}

pub enum NameEntryMsg {
    Show(String, Vec<String>),
    Cancel,
    Save,
    SetText(String),
}

impl Model for NameEntryModel {
    type Msg = NameEntryMsg;
    type Widgets = NameEntryWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for NameEntryModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        NameEntryModel {
            hidden: true,
            msg: String::default(),
            text: String::default(),
            existing: vec![],
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: NameEntryMsg,
        _components: &(),
        _sender: Sender<NameEntryMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            NameEntryMsg::Show(msg, existing) => {
                self.set_hidden(false);
                self.set_msg(msg);
                self.set_existing(existing);
                self.set_text(String::default());
            }
            NameEntryMsg::Cancel => self.hidden = true,
            NameEntryMsg::Save => {
                self.set_hidden(true);
                send!(parent_sender, AppMsg::AddNameAttr(None, self.text.clone()));
            }
            NameEntryMsg::SetText(s) => {
                self.text = s;
            }
        }
    }
}

#[relm4::widget(pub)]
impl Widgets<NameEntryModel, AppModel> for NameEntryWidgets {
    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_text: Some("Enter a new <name> value"),
            set_secondary_text: None,
            add_button: args!("Save", gtk::ResponseType::Accept),
            add_button: args!("Cancel", gtk::ResponseType::Cancel),
            connect_response(sender) => move |_, resp| {
                send!(sender, match resp {
                    gtk::ResponseType::Accept => NameEntryMsg::Save,
                    gtk::ResponseType::Cancel => NameEntryMsg::Cancel,
                    _ => unreachable!(),
                });
            },
        }
    }

    additional_fields! {
        textentry: gtk::Entry,
        msgbuf: gtk::EntryBuffer,
    }

    fn pre_init() {
        view! {
            textentry = gtk::Entry {
                set_margin_start: 20,
                set_margin_end: 20,
                set_hexpand: true,
                set_buffer: msgbuf = &gtk::EntryBuffer {
                    connect_text_notify(sender) => move |x| {
                        send!(sender, NameEntryMsg::SetText(x.text()));
                    },
                }
            }
        }
    }

    fn post_init() {
        dialog.content_area().append(&textentry);
        let accept_widget = dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        accept_widget.set_sensitive(false);
    }

    fn pre_view() {
        let accept_widget = dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        if model.text.is_empty() || model.existing.contains(&model.text) {
            accept_widget.set_css_classes(&[]);
            accept_widget.set_sensitive(false);
        } else {
            accept_widget.set_css_classes(&["suggested-action"]);
            accept_widget.set_sensitive(true);
        }
        if model.hidden {
            msgbuf.set_text("");
        }
    }
}
