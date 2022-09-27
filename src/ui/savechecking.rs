use super::window::AppMsg;
use crate::ui::optionpage::OptPageMsg;
use adw::prelude::*;
use log::{debug, info};
use relm4::*;
use sourceview5::prelude::*;
use std::{process::Command, path::Path};

pub struct SaveAsyncHandler;

#[derive(Debug)]
pub enum SaveAsyncHandlerMsg {
    SaveCheck(String, String, String, Vec<String>),
}

impl Worker for SaveAsyncHandler {
    type InitParams = ();
    type Input = SaveAsyncHandlerMsg;
    type Output = OptPageMsg;

    fn init(_params: Self::InitParams, _sender: relm4::ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SaveAsyncHandlerMsg::SaveCheck(opt, refopt, conf, alloptions) => {
                info!("Recived SaveCheck message");
                debug!("opt: {}\nrefopt: {}", opt, refopt);
                // For users.users.<name>.autoSubUidGidRange
                // (options.users.users.type.getSubOptions []).autoSubUidGidRange.type.check
                let checkcmd = {
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
                        } else if alloptions.contains(&p[..i].join(".")) && i + 1 < p.len()
                        /* Check if option exists */
                        {
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
                let output = if Path::new("/nix/var/nix/profiles/per-user/root/channels/nixos").exists() {
                    Command::new("nix-instantiate")
                    .arg("--eval")
                    .arg("--expr")
                    .arg(format!(
                        "with import <nixpkgs/nixos> {{}}; {} ({})",
                        checkcmd, conf
                    ))
                    .output()
                } else {
                    match Command::new("nix")
                        .arg("eval")
                        .arg("nixpkgs#path")
                        .output()
                    {
                        Ok(nixpath) => {
                            let nixospath = format!("{}/nixos", String::from_utf8_lossy(&nixpath.stdout).trim());
                            Command::new("nix-instantiate")
                                .arg("--eval")
                                .arg("--expr")
                                .arg(format!(
                                    "with import {} {{}}; {} ({})",
                                    nixospath, checkcmd, conf
                                ))
                                .output()
                        }
                        Err(e) => Err(e)
                    }
                };
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
                    Err(e) => (false, e.to_string()),
                };
                sender.output(OptPageMsg::DoneSaving(b, s));
            }
        }
    }
}

pub struct SaveErrorModel {
    hidden: bool,
    msg: String,
    scheme: Option<sourceview5::StyleScheme>,
}

#[derive(Debug)]
pub enum SaveErrorMsg {
    Show(String),
    SaveError,
    Reset,
    Cancel,
    SetScheme(String),
}

#[relm4::component(pub)]
impl SimpleComponent for SaveErrorModel {
    type InitParams = gtk::Window;
    type Input = SaveErrorMsg;
    type Output = AppMsg;
    type Widgets = SaveErrorWidgets;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_text: Some("Invalid configuration"),
            set_secondary_text: Some("Please fix the errors and try again."),
            #[watch]
            set_default_height: -1,
            set_default_width: 500,
            add_button: ("Keep changes", gtk::ResponseType::DeleteEvent),
            add_button: ("Reset", gtk::ResponseType::Reject),
            add_button: ("Edit", gtk::ResponseType::Cancel),
            connect_response[sender] => move |_, resp| {
                sender.input(match resp {
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

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SaveErrorModel {
            hidden: true,
            msg: String::default(),
            scheme: None,
        };

        view! {
            frame = gtk::Frame {
                set_margin_start: 20,
                set_margin_end: 20,
                gtk::ScrolledWindow {
                    set_vscrollbar_policy: gtk::PolicyType::Never,
                    sourceview5::View {
                        set_vexpand: true,
                        set_editable: false,
                        set_cursor_visible: false,
                        set_monospace: true,
                        set_top_margin: 5,
                        set_bottom_margin: 5,
                        set_left_margin: 5,
                        #[wrap(Some)]
                        set_buffer: msgbuf = &sourceview5::Buffer {
                            #[track(model.scheme)]
                            set_style_scheme: model.scheme.as_ref(),
                        },
                    }
                }
            }
        }

        let widgets = view_output!();
        widgets.dialog.content_area().append(&widgets.frame);

        let accept_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::DeleteEvent)
            .expect("No button for accept response set");
        accept_widget.add_css_class("destructive-action");
        ComponentParts { model, widgets }
    }

    fn pre_view() {
        msgbuf.set_text(&model.msg);
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SaveErrorMsg::Show(s) => {
                self.hidden = false;
                self.msg = s;
            }
            SaveErrorMsg::SaveError => {
                self.hidden = true;
                sender.output(AppMsg::SaveWithError);
            }
            SaveErrorMsg::Reset => {
                self.hidden = true;
                sender.output(AppMsg::SaveErrorReset);
            }
            SaveErrorMsg::Cancel => self.hidden = true,
            SaveErrorMsg::SetScheme(scheme) => {
                self.scheme = sourceview5::StyleSchemeManager::default().scheme(&scheme);
            }
        }
    }
}
