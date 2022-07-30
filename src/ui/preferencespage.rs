use super::window::AppMsg;
use adw::prelude::*;
use relm4::*;

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

#[derive(Debug)]
pub enum PrefMsg {
    Show(String, Option<String>),
    Close,
    UpdateConfPath(String),
    UpdateConfPathFC(String),
    UpdateFlake(Option<String>),
    FlakeSelected(bool),
    SetBusy(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for PrefModel {
    type InitParams = gtk::Window;
    type Input = PrefMsg;
    type Output = AppMsg;
    type Widgets = PrefWidgets;

    view! {
        window = adw::PreferencesWindow {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_search_enabled: false,
            connect_close_request[sender] => move |_| {
                sender.input(PrefMsg::Close);
                gtk::Inhibit(true)
            },
            #[watch]
            set_sensitive: !model.busy,
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
                                #[track(model.changed(PrefModel::filechooser()))]
                                set_text: &model.filechooser,
                                set_secondary_icon_name: Some("folder-documents-symbolic"),
                                set_secondary_icon_activatable: true,
                                connect_changed[sender] => move |x| {
                                    sender.input(PrefMsg::UpdateConfPath(x.text().to_string()));
                                }
                            }
                        },
                        append = &adw::ActionRow {
                            set_title: "Flake",
                            set_subtitle: "Whether the system is configured with Nix Flakes. If you don't know what this is, leave as false.",
                            set_selectable: false,
                            set_activatable: false,
                            add_suffix: flakebtn = &gtk::CheckButton {
                                #[track(model.changed(PrefModel::flakeselected()))]
                                set_active: model.flakeselected,
                                connect_toggled[sender, flakeentry] => move |x| {
                                    sender.input(PrefMsg::FlakeSelected(x.is_active()));
                                    if x.is_active() {
                                        sender.input(PrefMsg::UpdateFlake(Some(flakeentry.text().to_string())));
                                    } else {
                                        sender.input(PrefMsg::UpdateFlake(None));
                                    }
                                }
                            }
                        },
                        append = &adw::ActionRow {
                            #[watch]
                            set_visible: model.flakeselected,
                            set_title: "Flake Argument",
                            set_subtitle: "Argument passed into <tt>nixos-rebuild</tt>.\nWill use \"<tt>--flake <i>this_option</i></tt>\" if set.",
                            set_selectable: false,
                            set_activatable: false,
                            add_suffix: flakeentry = &gtk::Entry {
                                set_valign: gtk::Align::Center,
                                set_width_chars: 20,
                                #[track(model.changed(PrefModel::flakeselected()) && model.flake.is_some())]
                                set_text: match model.flake.as_ref() {
                                    Some(s) => s,
                                    None => "",
                                },
                                connect_changed[sender] => move |x| {
                                    sender.input(PrefMsg::UpdateFlake(Some(x.text().to_string())));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PrefModel {
            hidden: true,
            confpath: String::default(),
            flake: None,
            origconfpath: String::default(),
            origflake: None,
            flakeselected: false,
            filechooser: String::default(),
            busy: false,
            tracker: 0,
        };

        let widgets = view_output!();

        let filechooser = gtk::FileChooserDialog::new(
            Some("Select a configuration file"),
            Some(&widgets.window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        filechooser.set_default_size(1500, 700);
        filechooser
            .widget_for_response(gtk::ResponseType::Accept)
            .unwrap()
            .add_css_class("suggested-action");
        {
            let sender = sender.clone();
            widgets.confentry.connect_icon_release(move |_, _| {
                sender.input(PrefMsg::SetBusy(true));
                let sender = sender.clone();
                filechooser.run_async(move |obj, r| {
                    obj.hide();
                    obj.destroy();
                    sender.input(PrefMsg::SetBusy(false));
                    if let Some(x) = obj.file() {
                        if let Some(y) = x.path() {
                            if r == gtk::ResponseType::Accept {
                                sender.input(PrefMsg::UpdateConfPathFC(
                                    y.to_string_lossy().to_string(),
                                ));
                            }
                        }
                    }
                });
            });
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
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
            }
            PrefMsg::Close => {
                if !self.confpath.eq(&self.origconfpath) || !self.flake.eq(&self.origflake) {
                    sender.output(AppMsg::SetConfPath(
                        self.confpath.clone(),
                        self.flake.clone(),
                    ));
                }
                self.hidden = true;
            }
            PrefMsg::UpdateConfPath(s) => {
                self.set_confpath(s);
            }
            PrefMsg::UpdateConfPathFC(s) => {
                self.set_filechooser(s.clone());
                sender.input(PrefMsg::UpdateConfPath(s));
            }
            PrefMsg::UpdateFlake(s) => {
                self.set_flake(s);
            }
            PrefMsg::FlakeSelected(b) => {
                self.set_flakeselected(b);
            }
            PrefMsg::SetBusy(b) => {
                self.set_busy(b);
                sender.output(AppMsg::SetBusy(b));
            }
        }
    }
}

#[tracker::track]
pub struct WelcomeModel {
    hidden: bool,
    confpath: String,
}

#[derive(Debug)]
pub enum WelcomeMsg {
    Show,
    Close,
    UpdateConfPath(String),
}

#[relm4::component(pub)]
impl SimpleComponent for WelcomeModel {
    type InitParams = gtk::Window;
    type Input = WelcomeMsg;
    type Output = AppMsg;
    type Widgets = WelcomeWidgets;

    view! {
        window = adw::Window {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                gtk::Box {
                    set_valign: gtk::Align::Center,
                    set_vexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        gtk::Label {
                            add_css_class: "title-1",
                            set_text: "Welcome the NixOS Configuration Editor",
                            set_justify: gtk::Justification::Center,
                        },
                        gtk::Label {
                            add_css_class: "dim-label",
                            set_text: "If your configuration file is not in the default location, you can change it here.",
                        },
                    },
                    #[name(confentry)]
                    gtk::Entry {
                        set_width_chars: 20,
                        #[track(model.changed(WelcomeModel::confpath()))]
                        set_text: &model.confpath,
                        set_secondary_icon_name: Some("folder-documents-symbolic"),
                        set_secondary_icon_activatable: true,
                        set_hexpand: false,
                        set_halign: gtk::Align::Center,
                    },
                    #[name(btn)]
                    gtk::Button {
                        add_css_class: "pill",
                        add_css_class: "suggested-action",
                        set_label: "Continue",
                        set_hexpand: false,
                        set_halign: gtk::Align::Center,
                        connect_clicked[sender] => move |_| {
                            sender.input(WelcomeMsg::Close);
                        },
                    }
                }
            }
        }
    }

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = WelcomeModel {
            hidden: true,
            confpath: String::default(), // parent_window.configpath.to_string(),
            tracker: 0,
        };

        let widgets = view_output!();

        let filechooser = gtk::FileChooserDialog::new(
            Some("Select a configuration file"),
            Some(&widgets.window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );
        filechooser.set_default_size(1500, 700);
        filechooser
            .widget_for_response(gtk::ResponseType::Accept)
            .unwrap()
            .add_css_class("suggested-action");
        {
            let sender = sender.clone();
            widgets.confentry.connect_icon_release(move |_, _| {
                let sender = sender.clone();
                filechooser.run_async(move |obj, _| {
                    obj.hide();
                    obj.destroy();
                    if let Some(x) = obj.file() {
                        if let Some(y) = x.path() {
                            sender
                                .input(WelcomeMsg::UpdateConfPath(y.to_string_lossy().to_string()));
                        }
                    }
                });
            });
        }
        widgets.btn.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        self.reset();
        match msg {
            WelcomeMsg::Show => {
                self.hidden = false;
            }
            WelcomeMsg::Close => {
                sender.output(AppMsg::SetConfPath(self.confpath.clone(), None));
                self.hidden = true;
            }
            WelcomeMsg::UpdateConfPath(s) => {
                self.set_confpath(s);
            }
        }
    }
}
