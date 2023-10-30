use super::window::AppMsg;
use adw::prelude::*;
use log::info;
use nix_data::config::configfile::NixDataConfig;
use relm4::*;
use relm4_components::open_dialog::*;
use std::path::{Path, PathBuf};

pub struct WelcomeModel {
    hidden: bool,
    confpath: Option<PathBuf>,
    flakepath: Option<PathBuf>,
    conf_dialog: Controller<OpenDialog>,
    flake_dialog: Controller<OpenDialog>,
}

#[derive(Debug)]
pub enum WelcomeMsg {
    Show,
    Close,
    UpdateConfPath(PathBuf),
    UpdateFlakePath(PathBuf),
    ClearFlakePath,
    OpenConf,
    OpenFlake,
    Ignore,
}

#[relm4::component(pub)]
impl SimpleComponent for WelcomeModel {
    type Init = gtk::Window;
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
                            set_text: "Welcome the NixOS Configuration Editor!",
                            set_justify: gtk::Justification::Center,
                        },
                        gtk::Label {
                            add_css_class: "dim-label",
                            set_text: "If your configuration file is not in the default location, you can change it here.",
                        },
                    },
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_halign: gtk::Align::Fill,
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::ActionRow {
                            set_title: "Configuration file",
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,
                                    gtk::Image {
                                        set_icon_name: Some("document-open-symbolic"),
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: {
                                            if let Some(path) = &model.confpath {
                                                let x = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
                                                if x.is_empty() {
                                                    "(None)"
                                                } else {
                                                    x
                                                }
                                            } else {
                                                "(None)"
                                            }
                                        }
                                    }
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(WelcomeMsg::OpenConf);
                                }
                            },
                        },
                    },
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_halign: gtk::Align::Fill,
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::ActionRow {
                            set_title: "Flake file",
                            set_subtitle: "If you are using flakes, you can specify the path to your flake.nix file here.",
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 5,
                                    gtk::Image {
                                        set_icon_name: Some("document-open-symbolic"),
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: {
                                            if let Some(path) = &model.flakepath {
                                                let x = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
                                                if x.is_empty() {
                                                    "(None)"
                                                } else {
                                                    x
                                                }
                                            } else {
                                                "(None)"
                                            }
                                        }
                                    }
                                },
                                connect_clicked[sender] => move |_| {
                                    sender.input(WelcomeMsg::OpenFlake);
                                }
                            },
                            add_suffix = &gtk::Button {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_icon_name: "user-trash-symbolic",
                                connect_clicked[sender] => move |_| {
                                    sender.input(WelcomeMsg::ClearFlakePath);
                                }
                            }
                        },
                    },
                    #[name(btn)]
                    gtk::Button {
                        #[watch]
                        set_sensitive: model.confpath.is_some(),
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
        parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let conf_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => WelcomeMsg::UpdateConfPath(path),
                OpenDialogResponse::Cancel => WelcomeMsg::Ignore,
            });

        let flake_dialog = OpenDialog::builder()
            .transient_for_native(root)
            .launch(OpenDialogSettings::default())
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => WelcomeMsg::UpdateFlakePath(path),
                OpenDialogResponse::Cancel => WelcomeMsg::Ignore,
            });

        let model = WelcomeModel {
            hidden: true,
            confpath: if Path::new("/etc/nixos/configuration.nix").exists() {
                Some(PathBuf::from("/etc/nixos/configuration.nix"))
            } else {
                None
            }, // parent_window.configpath.to_string(),
            flakepath: None,
            conf_dialog,
            flake_dialog,
        };

        let widgets = view_output!();

        widgets.btn.grab_focus();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            WelcomeMsg::Show => {
                self.hidden = false;
            }
            WelcomeMsg::Close => {
                if let Some(confpath) = &self.confpath {
                    let _ = sender.output(AppMsg::SetConfig(NixDataConfig {
                        systemconfig: Some(confpath.to_string_lossy().to_string()),
                        flake: self
                            .flakepath
                            .as_ref()
                            .map(|x| x.to_string_lossy().to_string()),
                        flakearg: None,
                        generations: None,
                    }));
                    self.hidden = true;
                }
            }
            WelcomeMsg::UpdateConfPath(s) => {
                info!("Set configuration path to {}", s.to_string_lossy());
                self.confpath = Some(s);
            }
            WelcomeMsg::UpdateFlakePath(s) => {
                info!("Set flake path to {}", s.to_string_lossy());
                self.flakepath = Some(s);
            }
            WelcomeMsg::ClearFlakePath => {
                info!("Clear flake path");
                self.flakepath = None;
            }
            WelcomeMsg::OpenConf => self.conf_dialog.emit(OpenDialogMsg::Open),
            WelcomeMsg::OpenFlake => self.flake_dialog.emit(OpenDialogMsg::Open),
            WelcomeMsg::Ignore => {}
        }
    }
}
