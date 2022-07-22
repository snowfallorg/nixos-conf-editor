use super::window::{AppModel, AppMsg};
use adw::prelude::*;
use relm4::{
    send, ComponentUpdate, MessageHandler, Model, RelmMsgHandler, Sender, WidgetPlus, Widgets,
};
use sourceview5::prelude::*;
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::*;
use std::{io::BufReader, process::Command};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};

#[tracker::track]
pub struct RebuildModel {
    hidden: bool,
    text: String,
    status: RebuildStatus,
    config: String,
    path: String,
    flake: Option<String>,
    scheme: Option<sourceview5::StyleScheme>,
}

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
}

#[derive(PartialEq)]
enum RebuildStatus {
    Building,
    Success,
    Error,
}

#[derive(relm4::Components)]
pub struct RebuildComponents {
    async_handler: RelmMsgHandler<RebuildAsyncHandler, RebuildModel>,
}

impl Model for RebuildModel {
    type Msg = RebuildMsg;
    type Widgets = RebuildWidgets;
    type Components = RebuildComponents;
}

impl ComponentUpdate<AppModel> for RebuildModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        RebuildModel {
            hidden: true,
            text: String::new(),
            status: RebuildStatus::Building,
            config: String::new(),
            path: String::new(),
            flake: None,
            scheme: None,
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: RebuildMsg,
        components: &RebuildComponents,
        sender: Sender<RebuildMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            RebuildMsg::Rebuild(f, path, flake) => {
                self.update_hidden(|x| *x = false);
                self.update_text(|x| x.clear());
                self.set_config(f.to_string());
                self.set_path(path.to_string());
                self.set_flake(flake.clone());
                self.set_status(RebuildStatus::Building);
                components
                    .async_handler
                    .sender()
                    .blocking_send(RebuildAsyncHandlerMsg::RunRebuild(f, path, flake))
                    .unwrap();
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
                components
                    .async_handler
                    .sender()
                    .blocking_send(RebuildAsyncHandlerMsg::WriteConfig(
                        self.config.to_string(),
                        self.path.to_string(),
                    ))
                    .unwrap();
                send!(sender, RebuildMsg::Close);
            }
            RebuildMsg::Reset => {
                send!(parent_sender, AppMsg::ResetConfig);
                send!(sender, RebuildMsg::Close);
            }
            RebuildMsg::Save => {
                send!(parent_sender, AppMsg::SaveConfig);
                send!(sender, RebuildMsg::Close);
            }
            RebuildMsg::Close => {
                self.update_hidden(|x| *x = true);
                self.update_text(|x| x.clear());
            }
            RebuildMsg::SetScheme(scheme) => {
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
        }
    }
}

#[relm4::widget(pub)]
impl Widgets<RebuildModel, AppModel> for RebuildWidgets {
    view! {
        dialog = adw::Window {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_default_width: track!(model.changed(RebuildModel::hidden()), 500),
            set_default_height: track!(model.changed(RebuildModel::hidden()), 200),//295),
            set_resizable: true,
            set_visible: watch!(!model.hidden),
            add_css_class: "dialog",
            add_css_class: "message",
            set_content = Some(&gtk::Box) {
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
                    set_child: scrollwindow = Some(&gtk::ScrolledWindow) {
                        set_max_content_height: 500,
                        set_min_content_height: 100,
                        set_child: outview = Some(&sourceview5::View) {
                            set_editable: false,
                            set_cursor_visible: false,
                            set_monospace: true,
                            set_top_margin: 5,
                            set_bottom_margin: 5,
                            set_left_margin: 5,
                            set_vexpand: true,
                            set_hexpand: true,
                            set_vscroll_policy: gtk::ScrollablePolicy::Minimum,
                            set_buffer: outbuf = Some(&sourceview5::Buffer) {
                                set_style_scheme: track!(model.changed(RebuildModel::scheme()), model.scheme.as_ref()),
                                set_text: track!(model.changed(RebuildModel::text()), &model.text),
                            }
                        }
                    }
                },
                append = &gtk::Box {
                    add_css_class: "dialog-action-area",
                    set_orientation: gtk::Orientation::Horizontal,
                    set_homogeneous: true,
                    set_visible: track!(model.changed(RebuildModel::status()), model.status != RebuildStatus::Building),
                    append = &gtk::Button {
                        set_label: "Close",
                        set_visible: track!(model.changed(RebuildModel::status()), model.status == RebuildStatus::Success),
                        connect_clicked(sender) => move |_| {
                            send!(sender, RebuildMsg::Save)
                        }
                    },
                    append = &gtk::Button {
                        add_css_class: "destructive-action",
                        set_label: "Save Anyways",
                        set_visible: track!(model.changed(RebuildModel::status()), model.status == RebuildStatus::Error),
                        connect_clicked(sender) => move |_| {
                            send!(sender, RebuildMsg::Save)
                        }
                    },
                    append = &gtk::Button {
                        set_label: "Reset Changes",
                        set_visible: track!(model.changed(RebuildModel::status()), model.status == RebuildStatus::Error),
                        connect_clicked(sender) => move |_| {
                            send!(sender, RebuildMsg::Reset)
                        }
                    },
                    append = &gtk::Button {
                        set_label: "Keep Editing",
                        set_visible: track!(model.changed(RebuildModel::status()), model.status == RebuildStatus::Error),
                        connect_clicked(sender) => move |_| {
                            send!(sender, RebuildMsg::KeepEditing)
                        }
                    }
                }
            }
        }
    }

    fn pre_view() {
        match model.status {
            RebuildStatus::Building => {
                self.statusstack.set_visible_child(&self.building);
            }
            RebuildStatus::Success => self.statusstack.set_visible_child(&self.success),
            RebuildStatus::Error => self.statusstack.set_visible_child(&self.error),
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
}

pub struct RebuildAsyncHandler {
    _rt: Runtime,
    pub sender: TokioSender<RebuildAsyncHandlerMsg>,
}

#[derive(Debug)]
pub enum RebuildAsyncHandlerMsg {
    RunRebuild(String, String, Option<String>),
    WriteConfig(String, String),
}

impl MessageHandler<RebuildModel> for RebuildAsyncHandler {
    type Msg = RebuildAsyncHandlerMsg;
    type Sender = TokioSender<RebuildAsyncHandlerMsg>;

    fn init(_parent_model: &RebuildModel, parent_sender: Sender<RebuildMsg>) -> Self {
        let (sender, mut rx) = channel::<RebuildAsyncHandlerMsg>(10);

        let rt = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_time()
            .build()
            .unwrap();

        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
                                },
                                Err(_) => {
                                    String::from("nce-helper")
                                }
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
                                    send!(parent_sender, RebuildMsg::UpdateText(line));
                                });
                            if cmd.wait().as_ref().unwrap().success() {
                                send!(parent_sender, RebuildMsg::FinishSuccess);
                            } else {
                                send!(parent_sender, RebuildMsg::FinishError(None));
                            }
                        }
                        RebuildAsyncHandlerMsg::WriteConfig(f, path) => {
                            let exe = match std::env::current_exe() {
                                Ok(mut e) => {
                                    e.pop();
                                    e.push("nce-helper");
                                    e.to_string_lossy().to_string()
                                },
                                Err(_) => {
                                    String::from("nce-helper")
                                }
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
                        }
                    }
                });
            }
        });

        RebuildAsyncHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}
