use super::window::{AppModel, AppMsg};
use adw::prelude::*;
use relm4::*;
use sourceview5::prelude::*;
use std::convert::identity;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::*;
use std::{io::BufReader, process::Command};

#[tracker::track]
pub struct RebuildModel {
    hidden: bool,
    text: String,
    status: RebuildStatus,
    config: String,
    path: String,
    flake: Option<String>,
    scheme: Option<sourceview5::StyleScheme>,
    #[tracker::no_eq]
    async_handler: WorkerController<RebuildAsyncHandler>,
}

#[derive(Debug)]
pub enum RebuildMsg {
    Rebuild(String, String, Option<String>),
    FinishSuccess,
    FinishError(Option<String>),
    UpdateText(String),
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
    type InitParams = gtk::Window;
    type Input = RebuildMsg;
    type Output = AppMsg;
    type Widgets = RebuildWidgets;

    view! {
        dialog = adw::Window {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[track(model.hidden)]
            set_default_width: 500,
            #[track(model.hidden)]
            set_default_height: 200,//295),
            set_resizable: true,
            #[watch]
            set_visible: !model.hidden,
            add_css_class: "dialog",
            add_css_class: "message",
            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                append: statusstack = &gtk::Stack {
                    set_margin_top: 20,
                    set_transition_type: gtk::StackTransitionType::Crossfade,
                    set_vhomogeneous: false,
                    add_child: building = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append: spinner = &gtk::Spinner {
                            set_spinning: true,
                            set_height_request: 60,
                        },
                        append = &gtk::Label {
                            set_label: "Building...",
                            add_css_class: "title-1",
                        },
                    },
                    add_child: success = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append = &gtk::Image {
                            add_css_class: "success",
                            set_icon_name: Some("object-select-symbolic"),
                            set_pixel_size: 128,
                        },
                        append = &gtk::Label {
                            set_label: "Done!",
                            add_css_class: "title-1",
                        },
                        append = &gtk::Label {
                            set_label: "Rebuild successful!",
                            add_css_class: "dim-label",
                        }
                    },
                    add_child: error = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append = &gtk::Image {
                            add_css_class: "error",
                            set_icon_name: Some("dialog-error-symbolic"),
                            set_pixel_size: 128,
                        },
                        append = &gtk::Label {
                            set_label: "Error!",
                            add_css_class: "title-1",
                        },
                        append = &gtk::Label {
                            set_label: "Rebuild failed! See below for error message.",
                            add_css_class: "dim-label",
                        }
                    }
                },
                append = &gtk::Frame {
                    set_margin_all: 20,
                    #[wrap(Some)]
                    set_child: scrollwindow = &gtk::ScrolledWindow {
                        set_max_content_height: 500,
                        set_min_content_height: 100,
                        #[wrap(Some)]
                        set_child: outview = &sourceview5::View {
                            set_editable: false,
                            set_cursor_visible: false,
                            set_monospace: true,
                            set_top_margin: 5,
                            set_bottom_margin: 5,
                            set_left_margin: 5,
                            set_vexpand: true,
                            set_hexpand: true,
                            set_vscroll_policy: gtk::ScrollablePolicy::Minimum,
                            #[wrap(Some)]
                            set_buffer: outbuf = &sourceview5::Buffer {
                                #[track(model.changed(RebuildModel::scheme()))]
                                set_style_scheme: model.scheme.as_ref(),
                                #[track(model.changed(RebuildModel::text()))]
                                set_text: &model.text,
                            }
                        }
                    }
                },
                append = &gtk::Box {
                    add_css_class: "dialog-action-area",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,
                    #[track(model.changed(RebuildModel::status()))]
                    set_visible: model.status != RebuildStatus::Building,
                    append = &gtk::Button {
                        set_label: "Close",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Success,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Save)
                        }
                    },
                    append = &gtk::Button {
                        add_css_class: "destructive-action",
                        set_label: "Save Anyways",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Error,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Save)
                        }
                    },
                    append = &gtk::Button {
                        set_label: "Reset Changes",
                        #[track(model.changed(RebuildModel::status()))]
                        set_visible: model.status == RebuildStatus::Error,
                        connect_clicked[sender] => move |_| {
                            sender.input(RebuildMsg::Reset)
                        }
                    },
                    append = &gtk::Button {
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

    fn post_view() {
        let adj = scrollwindow.vadjustment();
        if model.status == RebuildStatus::Building {
            adj.set_upper(adj.upper() + 20.0);
        }
        adj.set_value(adj.upper());
        if model.status != RebuildStatus::Building {
            outview.scroll_to_mark(&outview.buffer().get_insert(), 0.0, true, 0.0, 0.0);
            scrollwindow.hadjustment().set_value(0.0);
        }
    }

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let async_handler = RebuildAsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);

        let model = RebuildModel {
            hidden: true,
            text: String::new(),
            status: RebuildStatus::Building,
            config: String::new(),
            path: String::new(),
            flake: None,
            scheme: None,
            async_handler,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        self.reset();
        match msg {
            RebuildMsg::Rebuild(f, path, flake) => {
                self.update_hidden(|x| *x = false);
                self.update_text(|x| x.clear());
                self.set_config(f.to_string());
                self.set_path(path.to_string());
                self.set_flake(flake.clone());
                self.set_status(RebuildStatus::Building);
                self.async_handler
                    .emit(RebuildAsyncHandlerMsg::RunRebuild(f, path, flake));
            }
            RebuildMsg::UpdateText(s) => {
                let newtext = if self.text.is_empty() {
                    s
                } else {
                    format!("{}\n{}", self.text, s)
                };
                self.set_text(newtext);
            }
            RebuildMsg::FinishSuccess => {
                self.set_status(RebuildStatus::Success);
            }
            RebuildMsg::FinishError(msg) => {
                if let Some(s) = msg {
                    self.set_text(s)
                }
                self.update_hidden(|x| *x = false);
                self.set_status(RebuildStatus::Error);
            }
            RebuildMsg::KeepEditing => {
                self.async_handler.emit(RebuildAsyncHandlerMsg::WriteConfig(
                    self.config.to_string(),
                    self.path.to_string(),
                    false,
                ));
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Reset => {
                sender.output(AppMsg::ResetConfig);
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Save => {
                sender.output(AppMsg::SaveConfig);
                sender.input(RebuildMsg::Close);
            }
            RebuildMsg::Close => {
                self.update_hidden(|x| *x = true);
                self.update_text(|x| x.clear());
            }
            RebuildMsg::SetScheme(scheme) => {
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
            RebuildMsg::WriteConfigQuit(f, path) => {
                self.async_handler
                    .emit(RebuildAsyncHandlerMsg::WriteConfig(f, path, true));
            }
            RebuildMsg::Quit => {
                sender.output(AppMsg::Close);
            }
        }
    }
}

#[derive(Debug)]
pub enum RebuildAsyncHandlerMsg {
    RunRebuild(String, String, Option<String>),
    WriteConfig(String, String, bool),
}

pub struct RebuildAsyncHandler;

impl Worker for RebuildAsyncHandler {
    type InitParams = ();
    type Input = RebuildAsyncHandlerMsg;
    type Output = RebuildMsg;

    fn init(_params: Self::InitParams, _sender: &relm4::ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        match msg {
            RebuildAsyncHandlerMsg::RunRebuild(f, path, flake) => {
                let exe = match std::env::current_exe() {
                    Ok(mut e) => {
                        e.pop(); // root/bin
                        e.pop(); // root/
                        e.push("libexec"); // root/libexec
                        e.push("nce-helper");
                        let x = e.to_string_lossy().to_string();
                        if Path::new(&x).is_file() {
                            x
                        } else {
                            String::from("nce-helper")
                        }
                    }
                    Err(_) => String::from("nce-helper"),
                };

                let mut writecmd = Command::new("pkexec")
                    .arg(&exe)
                    .arg("config")
                    .arg("--output")
                    .arg(&path)
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

                let mut cmd = if let Some(x) = flake {
                    Command::new("pkexec")
                        .arg(&exe)
                        .arg("rebuild")
                        .arg("--")
                        .arg("switch")
                        .arg("--flake")
                        .arg(x)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run nixos-rebuild")
                } else {
                    Command::new("pkexec")
                        .arg(&exe)
                        .arg("rebuild")
                        .arg("--")
                        .arg("switch")
                        .arg("-I")
                        .arg(format!("nixos-config={}", path))
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run nixos-rebuild")
                };

                let stderr = cmd.stderr.as_mut().unwrap();
                let reader = BufReader::new(stderr);

                reader
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| {
                        sender.output(RebuildMsg::UpdateText(line));
                    });
                if cmd.wait().as_ref().unwrap().success() {
                    sender.output(RebuildMsg::FinishSuccess);
                } else {
                    sender.output(RebuildMsg::FinishError(None));
                }
            }
            RebuildAsyncHandlerMsg::WriteConfig(f, path, quit) => {
                let exe = match std::env::current_exe() {
                    Ok(mut e) => {
                        e.pop(); // root/bin
                        e.pop(); // root/
                        e.push("libexec"); // root/libexec
                        e.push("nce-helper");
                        let x = e.to_string_lossy().to_string();
                        if Path::new(&x).is_file() {
                            x
                        } else {
                            String::from("nce-helper")
                        }
                    }
                    Err(_) => String::from("nce-helper"),
                };

                let mut writecmd = Command::new("pkexec")
                    .arg(&exe)
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
                    sender.output(RebuildMsg::Quit);
                }
            }
        }
    }
}
