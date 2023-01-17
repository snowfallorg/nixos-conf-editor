use super::window::AppMsg;
use adw::prelude::*;
use nix_data::config::configfile::NixDataConfig;
use relm4::*;
use relm4_components::open_dialog::*;
use std::path::PathBuf;

#[tracker::track]
#[derive(Debug)]
pub struct PreferencesPageModel {
    prefwindow: adw::PreferencesWindow,
    configpath: PathBuf,
    origconfigpath: PathBuf,
    flake: Option<PathBuf>,
    origflake: Option<PathBuf>,
    flakearg: Option<String>,
    origflakearg: Option<String>,
    #[tracker::no_eq]
    open_dialog: Controller<OpenDialog>,
    #[tracker::no_eq]
    flake_file_dialog: Controller<OpenDialog>,
    error: bool,
}

#[derive(Debug)]
pub enum PreferencesPageMsg {
    Show(NixDataConfig),
    ShowErr(NixDataConfig),
    Open,
    OpenFlake,
    SetConfigPath(PathBuf),
    SetFlakePath(Option<PathBuf>),
    SetFlakeArg(Option<String>),
    Close,
    Ignore,
}

#[relm4::component(pub)]
impl SimpleComponent for PreferencesPageModel {
    type Init = gtk::Window;
    type Input = PreferencesPageMsg;
    type Output = AppMsg;
    type Widgets = PreferencesPageWidgets;

    view! {
        adw::PreferencesWindow {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            set_search_enabled: false,
            connect_close_request[sender] => move |_| {
                sender.input(PreferencesPageMsg::Close);
                gtk::Inhibit(true)
            },
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    add = &adw::ActionRow {
                        set_title: "Configuration file",
                        add_suffix = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,
                            set_spacing: 10,
                            gtk::Button {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,
                                    gtk::Image {
                                        set_icon_name: Some("document-open-symbolic"),
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: {
                                            let x = model.configpath.to_str().unwrap_or_default();
                                            if x.is_empty() {
                                                "(None)"
                                            } else {
                                                x
                                            }
                                        }
                                    }
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(PreferencesPageMsg::Open);
                                }
                            },
                        }
                    },
                    add = &adw::ActionRow {
                        set_title: "Use nix flakes",
                        add_suffix = &gtk::Switch {
                            set_valign: gtk::Align::Center,
                            connect_state_set[sender] => move |_, b| {
                                if b {
                                    sender.input(PreferencesPageMsg::SetFlakePath(Some(PathBuf::new())));
                                } else {
                                    sender.input(PreferencesPageMsg::SetFlakePath(None));
                                    sender.input(PreferencesPageMsg::SetFlakeArg(None));
                                }
                                gtk::Inhibit(false)
                            } @switched,
                            #[track(model.changed(PreferencesPageModel::flake()))]
                            #[block_signal(switched)]
                            set_state: model.flake.is_some()
                        }
                    },
                    add = &adw::ActionRow {
                        set_title: "Flake file",
                        #[watch]
                        set_visible: model.flake.is_some(),
                        add_suffix = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,
                            set_spacing: 10,
                            gtk::Button {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,
                                    gtk::Image {
                                        set_icon_name: Some("document-open-symbolic"),
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: {
                                            let x = if let Some(f) = &model.flake {
                                                f.file_name().unwrap_or_default().to_str().unwrap_or_default()
                                            } else {
                                                ""
                                            };
                                            if x.is_empty() {
                                                "(None)"
                                            } else {
                                                x
                                            }
                                        }
                                    }
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(PreferencesPageMsg::OpenFlake);
                                }
                            },
                        }
                    },
                    add = &adw::EntryRow {
                        #[watch]
                        set_visible: model.flake.is_some(),
                        set_title: "Flake arguments (--flake path/to/flake.nix#<THIS ENTRY>)",
                        connect_changed[sender] => move |x| {
                            sender.input(PreferencesPageMsg::SetFlakeArg({
                                let text = x.text().to_string();
                                if text.is_empty() {
                                    None
                                } else {
                                    Some(text)
                                }}));
                        } @flakeentry,
                        #[track(model.changed(PreferencesPageModel::flake()))]
                        #[block_signal(flakeentry)]
                        set_text: model.flakearg.as_ref().unwrap_or(&String::new())
                    }

                }
            }
        }
    }

    fn init(
        parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => PreferencesPageMsg::SetConfigPath(path),
                OpenDialogResponse::Cancel => PreferencesPageMsg::Ignore,
            });
        let flake_file_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => PreferencesPageMsg::SetFlakePath(Some(path)),
                OpenDialogResponse::Cancel => PreferencesPageMsg::Ignore,
            });
        let model = PreferencesPageModel {
            prefwindow: root.clone(),
            configpath: PathBuf::new(),
            origconfigpath: PathBuf::new(),
            flake: None,
            origflake: None,
            flakearg: None,
            origflakearg: None,
            open_dialog,
            flake_file_dialog,
            error: false,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            PreferencesPageMsg::Show(config) => {
                if let Some(systemconfig) = &config.systemconfig {
                    self.configpath = PathBuf::from(systemconfig);
                    self.origconfigpath = PathBuf::from(systemconfig);
                }
                self.set_flake(config.flake.as_ref().map(PathBuf::from));
                self.origflake = self.flake.clone();
                self.set_flakearg(config.flakearg);
                self.origflakearg = self.flakearg.clone();
                self.prefwindow.show();
                self.error = false;
            }
            PreferencesPageMsg::ShowErr(config) => {
                if let Some(systemconfig) = &config.systemconfig {
                    self.configpath = PathBuf::from(systemconfig);
                    self.origconfigpath = PathBuf::from(systemconfig);
                }
                self.set_flake(config.flake.as_ref().map(PathBuf::from));
                self.origflake = self.flake.clone();
                self.set_flakearg(config.flakearg);
                self.origflakearg = self.flakearg.clone();
                self.prefwindow.present();
                self.error = true;
            }
            PreferencesPageMsg::Open => self.open_dialog.emit(OpenDialogMsg::Open),
            PreferencesPageMsg::OpenFlake => self.flake_file_dialog.emit(OpenDialogMsg::Open),
            PreferencesPageMsg::SetConfigPath(path) => {
                self.configpath = path;
            }
            PreferencesPageMsg::SetFlakePath(path) => {
                self.flake = path;
            }
            PreferencesPageMsg::SetFlakeArg(arg) => {
                self.flakearg = arg;
            }
            PreferencesPageMsg::Close => {
                if !self.configpath.eq(&self.origconfigpath)
                    || !self.flake.eq(&self.origflake)
                    || !self.flakearg.eq(&self.origflakearg)
                    || self.error
                {
                    let _ = sender.output(AppMsg::SetConfig(NixDataConfig {
                        systemconfig: Some(self.configpath.to_string_lossy().to_string()),
                        flake: self.flake.as_ref().map(|x| x.to_string_lossy().to_string()),
                        flakearg: self.flakearg.clone(),
                    }));
                }
                self.prefwindow.hide();
            }
            _ => {}
        }
    }
}
