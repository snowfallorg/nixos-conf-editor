use super::window::AppMsg;
use crate::config::LIBEXECDIR;
use adw::prelude::*;
use gtk::{gio, glib};
use relm4::*;
use std::io::Write;
use std::process::Command;
use std::process::*;
use vte::{TerminalExt, TerminalExtManual};

#[tracker::track]
pub struct RebuildModel {
    hidden: bool,
    status: RebuildStatus,
    config: String,
    path: String,
    flake: Option<String>,
    scheme: Option<sourceview5::StyleScheme>,
    terminal: vte::Terminal,
}

#[derive(Debug)]
pub enum RebuildMsg {
    Rebuild(String, String, Option<String>),
    FinishSuccess,
    FinishError(Option<String>),
    WriteConfig(String, String, bool),
    KeepEditing,
    Reset,
    Save,
    Close,
    SetScheme(String),
    WriteConfigQuit(String, String),
    Quit,
}

#[derive(PartialEq)]
enum RebuildStatus {
    Building,
    Success,
    Error,
}

#[relm4::component(pub)]
impl SimpleComponent for RebuildModel {
    type Init = gtk::Window;
    type Input = RebuildMsg;
    type Output = AppMsg;
    type Widgets = RebuildWidgets;

    view! {
        dialog = adw::Window {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[track(model.changed(RebuildModel::hidden()))]
            set_default_width: 500,
            #[track(model.changed(RebuildModel::hidden()))]
            set_default_height: 200,//295),
            set_resizable: true,
            #[watch]
            set_visible: !model.hidden,
            add_css_class: "dialog",
            add_css_class: "message",
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                #[name(statusstack)]
                gtk::Stack {
                    set_margin_top: 20,
                    set_transition_type: gtk::StackTransitionType::Crossfade,
                    set_vhomogeneous: false,
                    #[name(building)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        gtk::Spinner {
                            set_spinning: true,
                            set_height_request: 60,
                        },
                        gtk::Label {
                            set_label: "Building...",
                            add_css_class: "title-1",
                        },
                    },
                    #[name(success)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        gtk::Image {
                            add_css_class: "success",
                            set_icon_name: Some("object-select-symbolic"),
                            set_pixel_size: 128,
                        },
                        gtk::Label {
                            set_label: "Done!",
                            add_css_class: "title-1",
                        },
                        gtk::Label {
                            set_label: "Rebuild successful!",
                            add_css_class: "dim-label",
                        }
                    },
                    #[name(error)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        gtk::Image {
                            add_css_class: "error",
                            set_icon_name: Some("dialog-error-symbolic"),
                            set_pixel_size: 128,
                        },
                        gtk::Label {
                            set_label: "Error!",
                            add_css_class: "title-1",
                        },
                        gtk::Label {
                            set_label: "Rebuild failed! See below for error message.",
                            add_css_class: "dim-label",
                        }
                    }
                },
                gtk::Frame {
                    set_margin_all: 20,
                    #[name(scrollwindow)]
                    gtk::ScrolledWindow {
                        set_max_content_height: 500,
                        set_min_content_height: 100,
                        #[local_ref]
                        terminal -> vte::Terminal {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_input_enabled: false,
                            connect_child_exited[sender] => move |_term, status| {
                                if status == 0 {
                                    sender.input(RebuildMsg::FinishSuccess);
                                } else {
                                    sender.input(RebuildMsg::FinishError(None));
                                }
                            }
                        }
                    }
                },
                gtk::Box {
                    add_css_class: "dialog-action-area",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,
                    #[track(model.changed(RebuildModel::status()))]
                    set_visible: model.status != RebuildStatus::Building,
                    gtk::Button {
                        set_label: "Close",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Success,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Save)
                        }
                    },
                    gtk::Button {
                        add_css_class: "destructive-action",
                        set_label: "Save Anyways",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Error,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Save)
                        }
                    },
                    gtk::Button {
                        set_label: "Reset Changes",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Error,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Reset)
                        }
                    },
                    gtk::Button {
                        set_label: "Keep Editing",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Error,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::KeepEditing)
                        }
                    }
                }
            }
        }
    }

    fn pre_view() {
        match model.status {
            RebuildStatus::Building => {
                statusstack.set_visible_child(building);
            }
            RebuildStatus::Success => statusstack.set_visible_child(success),
            RebuildStatus::Error => statusstack.set_visible_child(error),
        }
    }

    fn init(
        parent_window: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RebuildModel {
            hidden: true,
            status: RebuildStatus::Building,
            config: String::new(),
            path: String::new(),
            flake: None,
            scheme: None,
            terminal: vte::Terminal::new(),
            tracker: 0,
        };

        let terminal = &model.terminal;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            RebuildMsg::Rebuild(f, path, flake) => {
                self.update_hidden(|x| *x = false);
                self.set_config(f.to_string());
                self.set_path(path.to_string());
                self.set_flake(flake.clone());
                self.set_status(RebuildStatus::Building);
                if let Some(flake) = flake {
                    self.terminal.spawn_async(
                        vte::PtyFlags::DEFAULT,
                        Some("/"),
                        &[
                            "/usr/bin/env",
                            "pkexec",
                            &format!("{}/nce-helper", LIBEXECDIR),
                            "write-rebuild",
                            "--content",
                            &f,
                            "--path",
                            &path,
                            "--",
                            "switch",
                            "--flake",
                            &flake,
                        ],
                        &[],
                        glib::SpawnFlags::DEFAULT,
                        || (),
                        -1,
                        gio::Cancellable::NONE,
                        |_, _, _| (),
                    );
                } else {
                    self.terminal.spawn_async(
                        vte::PtyFlags::DEFAULT,
                        Some("/"),
                        &[
                            "/usr/bin/env",
                            "pkexec",
                            &format!("{}/nce-helper", LIBEXECDIR),
                            "write-rebuild",
                            "--content",
                            &f,
                            "--path",
                            &path,
                            "--",
                            "switch",
                            "-I",
                            &format!("nixos-config={}", path),
                        ],
                        &[],
                        glib::SpawnFlags::DEFAULT,
                        || (),
                        -1,
                        gio::Cancellable::NONE,
                        |_, _, _| (),
                    );
                }
            }
            RebuildMsg::FinishSuccess => {
                self.set_status(RebuildStatus::Success);
            }
            RebuildMsg::FinishError(_msg) => {
                self.update_hidden(|x| *x = false);
                self.set_status(RebuildStatus::Error);
            }
            RebuildMsg::KeepEditing => {
                sender.input(RebuildMsg::WriteConfig(
                    self.config.to_string(),
                    self.path.to_string(),
                    false,
                ));
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Reset => {
                let _ = sender.output(AppMsg::ResetConfig);
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Save => {
                let _ = sender.output(AppMsg::SaveConfig);
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Close => {
                self.terminal.reset(true, true);
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &["/usr/bin/env", "clear"],
                    &[],
                    glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |_, _, _| (),
                );
                self.update_hidden(|x| *x = true);
            }
            RebuildMsg::SetScheme(scheme) => {
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
            RebuildMsg::WriteConfigQuit(f, path) => {
                sender.input(RebuildMsg::WriteConfig(f, path, true));
            }
            RebuildMsg::WriteConfig(f, path, quit) => {
                let mut writecmd = Command::new("pkexec")
                    .arg(&format!("{}/nce-helper", LIBEXECDIR))
                    .arg("config")
                    .arg("--output")
                    .arg(path)
                    .stdin(Stdio::piped())
                    .spawn()
                    .unwrap();
                writecmd
                    .stdin
                    .as_mut()
                    .ok_or("stdin not available")
                    .unwrap()
                    .write_all(f.as_bytes())
                    .unwrap();
                writecmd.wait().unwrap();
                if quit {
                    sender.input(RebuildMsg::Quit);
                }
            }
            RebuildMsg::Quit => {
                let _ = sender.output(AppMsg::Close);
            }
        }
    }
}
