use adw::prelude::*;
use relm4::*;

use super::window::{AppModel, AppMsg};

#[tracker::track]
pub struct PrefModel {
    hidden: bool,
    confpath: String,
    flake: Option<String>,
    origconfpath: String,
    origflake: Option<String>,
    flakeselected: bool,
    filechooser: String,
    busy: bool,
}

pub enum PrefMsg {
    Show(String, Option<String>),
    Close,
    UpdateConfPath(String),
    UpdateConfPathFC(String),
    UpdateFlake(Option<String>),
    FlakeSelected(bool),
    SetBusy(bool),
}

impl Model for PrefModel {
    type Msg = PrefMsg;
    type Widgets = PrefWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for PrefModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        PrefModel {
            hidden: true,
            confpath: String::default(),
            flake: None,
            origconfpath: String::default(),
            origflake: None,
            flakeselected: false,
            filechooser: String::default(),
            busy: false,
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: PrefMsg,
        _components: &(),
        sender: Sender<PrefMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            PrefMsg::Show(confpath, flake) => {
                self.set_confpath(confpath.clone());
                self.set_flake(flake.clone());
                self.set_flakeselected(flake.is_some());
                self.set_filechooser(confpath.clone());
                self.set_origconfpath(confpath);
                self.set_origflake(flake);
                self.hidden = false;
            },
            PrefMsg::Close => {
                if !self.confpath.eq(&self.origconfpath) || !self.flake.eq(&self.origflake) {
                    send!(parent_sender, AppMsg::SetConfPath(self.confpath.clone(), self.flake.clone()));
                }
                self.hidden = true;
            }
            PrefMsg::UpdateConfPath(s) => {
                self.set_confpath(s);
            }
            PrefMsg::UpdateConfPathFC(s) => {
                self.set_filechooser(s.clone());
                send!(sender, PrefMsg::UpdateConfPath(s));
            }
            PrefMsg::UpdateFlake(s) => {
                self.set_flake(s);
            }
            PrefMsg::FlakeSelected(b) => {
                self.set_flakeselected(b);
            }
            PrefMsg::SetBusy(b) => {
                self.set_busy(b);
                send!(parent_sender, AppMsg::SetBusy(b));
            }
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<PrefModel, AppModel> for PrefWidgets {
    view! {
        window = adw::PreferencesWindow {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_search_enabled: false,
            connect_close_request(sender) => move |_| {
                send!(sender, PrefMsg::Close);
                gtk::Inhibit(true)
            },
            set_sensitive: watch!(!model.busy),
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "Configuration",
                    add = &gtk::ListBox {
                        add_css_class: "boxed-list",
                        append = &adw::ActionRow {
                            set_title: "Location",
                            set_selectable: false,
                            set_activatable: false,
                            add_suffix: confentry = &gtk::Entry {
                                set_valign: gtk::Align::Center,
                                set_width_chars: 20,
                                set_text: track!(model.changed(PrefModel::filechooser()), &model.filechooser),
                                set_secondary_icon_name: Some("folder-documents-symbolic"),
                                set_secondary_icon_activatable: true,
                                connect_changed(sender) => move |x| {
                                    send!(sender, PrefMsg::UpdateConfPath(x.text().to_string()));
                                }
                            }
                        },
                        append = &adw::ActionRow {
                            set_title: "Flake",
                            set_subtitle: "Whether the system is configured with Nix Flakes. If you don't know what this is, leave as false.",
                            set_selectable: false,
                            set_activatable: false,
                            add_suffix: flakebtn = &gtk::CheckButton {
                                set_active: track!(model.changed(PrefModel::flakeselected()), model.flakeselected),
                                connect_toggled(sender, flakeentry) => move |x| {
                                    send!(sender, PrefMsg::FlakeSelected(x.is_active()));
                                    if x.is_active() {
                                        send!(sender, PrefMsg::UpdateFlake(Some(flakeentry.text().to_string())));
                                    } else {
                                        send!(sender, PrefMsg::UpdateFlake(None));
                                    }
                                }
                            }
                        },
                        append = &adw::ActionRow {
                            set_visible: watch!(model.flakeselected),
                            set_title: "Flake Argument",
                            set_subtitle: "Argument passed into <tt>nixos-rebuild</tt>.\nWill use \"<tt>--flake <i>this_option</i></tt>\" if set.",
                            set_selectable: false,
                            set_activatable: false,
                            add_suffix: flakeentry = &gtk::Entry {
                                set_valign: gtk::Align::Center,
                                set_width_chars: 20,
                                set_text: track!(model.changed(PrefModel::flakeselected()) && model.flake.is_some(), match model.flake.as_ref() {
                                    Some(s) => s,
                                    None => "",
                                }),
                                connect_changed(sender) => move |x| {
                                    send!(sender, PrefMsg::UpdateFlake(Some(x.text().to_string())));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn post_init() {
        let filechooser = gtk::FileChooserDialog::new(
            Some("Select a configuration file"),
            Some(&window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        filechooser.set_default_size(1500, 700);
        filechooser.widget_for_response(gtk::ResponseType::Accept).unwrap().add_css_class("suggested-action");
        confentry.connect_icon_release(move |_, _| {
            let sender = sender.clone();
            send!(sender, PrefMsg::SetBusy(true));
            filechooser.run_async(move |obj, r| {
                obj.hide();
                obj.destroy();
                send!(sender, PrefMsg::SetBusy(false));
                if let Some(x) = obj.file() {
                    if let Some(y) = x.path() {
                        if r == gtk::ResponseType::Accept {
                            send!(sender, PrefMsg::UpdateConfPathFC(y.to_string_lossy().to_string()));
                        }
                    }
                }
            });           
        });
    }
}


#[tracker::track]
pub struct WelcomeModel {
    hidden: bool,
    confpath: String,
}

pub enum WelcomeMsg {
    Show,
    Close,
    UpdateConfPath(String),
}

impl Model for WelcomeModel {
    type Msg = WelcomeMsg;
    type Widgets = WelcomeWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for WelcomeModel {
    fn init_model(parent_model: &AppModel) -> Self {
        WelcomeModel {
            hidden: true,
            confpath: parent_model.configpath.to_string(),
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: WelcomeMsg,
        _components: &(),
        _sender: Sender<WelcomeMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            WelcomeMsg::Show => {
                self.hidden = false;
            },
            WelcomeMsg::Close => {
                send!(parent_sender, AppMsg::SetConfPath(self.confpath.clone(), None));
                self.hidden = true;
            }
            WelcomeMsg::UpdateConfPath(s) => {
                self.set_confpath(s);
            }
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<WelcomeModel, AppModel> for WelcomeWidgets {
    view! {
        window = adw::Window {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_content = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &gtk::Box {
                    set_valign: gtk::Align::Center,
                    set_vexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append = &gtk::Label {
                            add_css_class: "title-1",
                            set_text: "Welcome the NixOS Configuration Editor",
                            set_justify: gtk::Justification::Center,
                        },
                        append = &gtk::Label {
                            add_css_class: "dim-label",
                            set_text: "If your configuration file is not in the default location, you can change it here.",
                        },
                    },
                    append: confentry = &gtk::Entry {
                        set_width_chars: 20,
                        set_text: track!(model.changed(WelcomeModel::confpath()), &model.confpath),
                        set_secondary_icon_name: Some("folder-documents-symbolic"),
                        set_secondary_icon_activatable: true,
                        set_hexpand: false,
                        set_halign: gtk::Align::Center,
                    },
                    append: btn = &gtk::Button {
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_label: "Continue",
                        set_hexpand: false,
                        set_halign: gtk::Align::Center,
                        connect_clicked(sender) => move |_| {
                            send!(sender, WelcomeMsg::Close);
                        },
                    }
                }
            }
        }
    }

    fn post_init() {
        let filechooser = gtk::FileChooserDialog::new(
            Some("Select a configuration file"),
            Some(&window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        filechooser.set_default_size(1500, 700);
        filechooser.widget_for_response(gtk::ResponseType::Accept).unwrap().add_css_class("suggested-action");
        confentry.connect_icon_release(move |_, _| {
            let sender = sender.clone();
            filechooser.run_async(move |obj, _| {
                obj.hide();
                obj.destroy();
                if let Some(x) = obj.file() {
                    if let Some(y) = x.path() {
                        send!(sender, WelcomeMsg::UpdateConfPath(y.to_string_lossy().to_string()));
                    }
                }
            });
        });
        btn.grab_focus();
    }
}