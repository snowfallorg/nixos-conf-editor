use super::window::*;
use adw::prelude::*;
use relm4::*;

pub struct NameEntryModel {
    hidden: bool,
    msg: String,
    existing: Vec<String>,
    text: String,
}

#[derive(Debug)]
pub enum NameEntryMsg {
    Show(String, Vec<String>),
    Cancel,
    Save,
    SetText(String),
}

#[relm4::component(pub)]
impl SimpleComponent for NameEntryModel {
    type Init = gtk::Window;
    type Input = NameEntryMsg;
    type Output = AppMsg;
    type Widgets = NameEntryWidgets;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_text: Some("Enter a new <name> value"),
            set_secondary_text: None,
            add_button: ("Save", gtk::ResponseType::Accept),
            add_button: ("Cancel", gtk::ResponseType::Cancel),
            connect_response[sender] => move |_, resp| {
                sender.input(match resp {
                    gtk::ResponseType::Accept => NameEntryMsg::Save,
                    gtk::ResponseType::Cancel => NameEntryMsg::Cancel,
                    _ => unreachable!(),
                });
            }
        }
    }

    additional_fields! {
        textentry: gtk::Entry,
        msgbuf: gtk::EntryBuffer,
    }

    fn init(
        parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = NameEntryModel {
            hidden: true,
            msg: String::default(),
            text: String::default(),
            existing: vec![],
        };

        view! {
            textentry = gtk::Entry {
                set_margin_start: 20,
                set_margin_end: 20,
                set_hexpand: true,
                set_buffer: msgbuf = &gtk::EntryBuffer {
                    connect_text_notify[sender] => move |x| {
                        sender.input(NameEntryMsg::SetText(x.text()));
                    },
                }
            }
        }

        let widgets = view_output!();

        widgets.dialog.content_area().append(&widgets.textentry);
        let accept_widget = widgets.dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        accept_widget.set_sensitive(false);

        ComponentParts { model, widgets }
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

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            NameEntryMsg::Show(msg, existing) => {
                self.hidden = false;
                self.msg = msg;
                self.existing = existing;
                self.text = String::default();
            }
            NameEntryMsg::Cancel => self.hidden = true,
            NameEntryMsg::Save => {
                self.hidden = true;
                sender.output(AppMsg::AddNameAttr(None, self.text.clone()));
            }
            NameEntryMsg::SetText(s) => {
                self.text = s;
            }
        }
    }
}
