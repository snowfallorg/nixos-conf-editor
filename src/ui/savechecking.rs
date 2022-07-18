
use crate::ui::optionpage::OptPageMsg;
use adw::prelude::*;
use relm4::*;
use sourceview5::prelude::*;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};
use std::process::Command;
use log::{debug, info, trace, warn};
use super::optionpage::*;
use super::window::{AppModel, AppMsg};

pub struct SaveAsyncHandler {
    _rt: Runtime,
    pub sender: TokioSender<SaveAsyncHandlerMsg>,
}

#[derive(Debug)]
pub enum SaveAsyncHandlerMsg {
    SaveCheck(String, String, String, Vec<String>),
}

impl MessageHandler<OptPageModel> for SaveAsyncHandler {
    type Msg = SaveAsyncHandlerMsg;
    type Sender = TokioSender<SaveAsyncHandlerMsg>;

    fn init(_parent_model: &OptPageModel, parent_sender: Sender<OptPageMsg>) -> Self {
        let (sender, mut rx) = channel::<SaveAsyncHandlerMsg>(10);

        let rt = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_time()
            .build()
            .unwrap();

        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    match msg {
                        SaveAsyncHandlerMsg::SaveCheck(opt, refopt, conf, alloptions) => {
                            info!("Recived SaveCheck message");
                            debug!("opt: {}\nrefopt: {}", opt, refopt);
                            // For users.users.<name>.autoSubUidGidRange
                            // (options.users.users.type.getSubOptions []).autoSubUidGidRange.type.check
                            let checkcmd =  {
                                let p = refopt.split('.').collect::<Vec<_>>();
                                let mut r: Vec<Vec<String>> = vec![vec![]];
                                let mut indexvec: Vec<usize> = vec![];
                                let mut j = 0;
                                for i in 0..p.len() {
                                    if p[i] == "*" || p[i] == "<name>" {
                                        r.push(vec![]);
                                        if let Ok(x) = opt.split('.').collect::<Vec<_>>()[i].parse::<usize>() {
                                            indexvec.push(x);
                                        }
                                        j += 1;
                                    } else if alloptions.contains(&p[..i].join(".")) && i+1 < p.len() /* Check if option exists */ {
                                        r.push(vec![]);
                                        j += 1;
                                        r[j].push(p[i].to_string());
                                    } else {
                                        r[j].push(p[i].to_string());
                                    }
                                }
                                let mut s = format!("options.{}", r[0].join("."));
                                for y in r[1..].iter() {
                                    s = format!("({}.type.getSubOptions []).{}", s, y.join("."));
                                }
                                format!("{}.type.check", s)
                            };
                            let output = Command::new("nix-instantiate")
                                    .arg("--eval")
                                    .arg("--expr")
                                    .arg(format!("with import <nixpkgs/nixos> {{}}; {} ({})", checkcmd, conf))
                                    .output();
                                let (b, s) = match output {
                                Ok(output) => {
                                    if output.status.success() {
                                        let output = String::from_utf8(output.stdout).unwrap();
                                        (true, output)
                                    } else {
                                        let output = String::from_utf8(output.stderr).unwrap();
                                        (false, output)
                                    }
                                }
                                Err(e) => {
                                    (false, e.to_string())
                                }
                                };
                                send!(parent_sender, OptPageMsg::DoneSaving(b, s));
                        }
                    }
                });
            }
        });

        SaveAsyncHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}


#[tracker::track]
pub struct SaveErrorModel {
    hidden: bool,
    msg: String,
    scheme: Option<sourceview5::StyleScheme>,
}

pub enum SaveErrorMsg {
    Show(String),
    SaveError,
    Reset,
    Cancel,
    SetScheme(String),
}

impl Model for SaveErrorModel {
    type Msg = SaveErrorMsg;
    type Widgets = SaveErrorWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for SaveErrorModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        SaveErrorModel {
            hidden: true,
            msg: String::default(),
            scheme: None,
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: SaveErrorMsg,
        _components: &(),
        _sender: Sender<SaveErrorMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            SaveErrorMsg::Show(s) => {
                self.hidden = false;
                self.msg = s;
            },
            SaveErrorMsg::SaveError => {
                self.hidden = true;
                parent_sender.send(AppMsg::SaveWithError).unwrap();
            },
            SaveErrorMsg::Reset => {
                self.hidden = true;
                parent_sender.send(AppMsg::SaveErrorReset).unwrap();
            },
            SaveErrorMsg::Cancel => self.hidden = true,
            SaveErrorMsg::SetScheme(scheme) => {
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<SaveErrorModel, AppModel> for SaveErrorWidgets {
    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_text: Some("Invalid configuration"),
            set_secondary_text: Some("Please fix the errors and try again."),
            set_default_height: watch!(-1),
            set_default_width: 500,
            add_button: args!("Keep changes", gtk::ResponseType::DeleteEvent),
            add_button: args!("Reset", gtk::ResponseType::Reject),
            add_button: args!("Edit", gtk::ResponseType::Cancel),
            connect_response(sender) => move |_, resp| {
                send!(sender, match resp {
                    gtk::ResponseType::DeleteEvent => SaveErrorMsg::SaveError,
                    gtk::ResponseType::Reject => SaveErrorMsg::Reset,
                    gtk::ResponseType::Cancel => SaveErrorMsg::Cancel,
                    _ => unreachable!(),
                });
            },
        }
    }

    additional_fields! {
        frame: gtk::Frame,
        msgbuf: sourceview5::Buffer,
    }
    
    fn pre_init() {
        view! {
            frame = gtk::Frame {
                set_margin_start: 20,
                set_margin_end: 20,
                set_child = Some(&gtk::ScrolledWindow) {
                    set_vscrollbar_policy: gtk::PolicyType::Never,
                    set_child = Some(&sourceview5::View) {
                        set_vexpand: true,
                        set_editable: false,
                        set_cursor_visible: false,
                        set_monospace: true,
                        set_top_margin: 5,
                        set_bottom_margin: 5,
                        set_left_margin: 5,
                        set_buffer: msgbuf = Some(&sourceview5::Buffer) {
                            set_style_scheme: track!(model.changed(SaveErrorModel::scheme()), model.scheme.as_ref()),
                        },
                    }
                }
            }
        }
    }

    fn post_init() {
        dialog.content_area().append(&frame);

        let accept_widget = dialog
            .widget_for_response(gtk::ResponseType::DeleteEvent)
            .expect("No button for accept response set");
        accept_widget.add_css_class("destructive-action");
    }

    fn pre_view() {
        msgbuf.set_text(&model.msg);
    }
}
