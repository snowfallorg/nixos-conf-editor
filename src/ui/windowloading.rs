use super::window::{AppMsg, LoadValues};
use crate::parse::cache::checkcache;
use crate::parse::config::parseconfig;
use crate::parse::options::read;
use crate::parse::preferences::{NceConfig, getconfig, editconfig};
use relm4::adw::prelude::*;
use relm4::*;
use std::error::Error;
use std::path::Path;

pub struct WindowAsyncHandler;

#[derive(Debug)]
pub enum WindowAsyncHandlerMsg {
    RunWindow(String),
    GetConfigPath(Option<NceConfig>),
    SetConfig(NceConfig),
}

impl Worker for WindowAsyncHandler {
    type InitParams = ();
    type Input = WindowAsyncHandlerMsg;
    type Output = AppMsg;

    fn init(_params: Self::InitParams, _sender: relm4::ComponentSender<Self>) -> Self {
        Self
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            WindowAsyncHandlerMsg::RunWindow(path) => {
                match checkcache() {
                    Ok(_) => {}
                    Err(_) => {
                        sender.output(AppMsg::LoadError(
                            String::from("Could not load cache"),
                            String::from(
                                "Try connecting to the internet or launching the application again",
                            ),
                        ));
                        return;
                    }
                }

                let (data, tree) = match read() {
                    Ok(x) => x,
                    Err(_) => {
                        sender.output(AppMsg::LoadError(
                            String::from("Could not load options"),
                            String::from("Try launching the application again"),
                        ));
                        return;
                    }
                };

                let conf = match parseconfig(&path) {
                    Ok(x) => x,
                    Err(_) => {
                        sender.output(AppMsg::LoadError(
                            String::from("Error loading configuration file"),
                            format!("<tt>{}</tt> may be an invalid configuration file", path),
                        ));
                        return;
                    }
                };
                sender.output(AppMsg::InitialLoad(LoadValues { data, tree, conf }))
            }
            WindowAsyncHandlerMsg::GetConfigPath(cfg) => {
                if let Some(config) = cfg {
                    if Path::new(&config.systemconfig).exists() {
                        if let Some(flakepath) = &config.flake {
                            if !Path::new(flakepath).exists() {
                                sender.output(AppMsg::Welcome);
                                return;
                            }
                        }
                        sender.output(AppMsg::SetConfig(config));
                    } else {
                        sender.output(AppMsg::Welcome);
                        return;
                    }
                } else {
                    sender.output(AppMsg::Welcome);
                    return;
                }

                // if !Path::new(&cfg.systemconfig).exists() {
                    // sender.output(AppMsg::Welcome);
                    // return;
                // }
                
                // match configvalues() {
                //     Ok((x, y)) => sender.output(AppMsg::SetConfPath(x, y)),
                //     Err(_) => sender.output(AppMsg::LoadError(
                //         String::from("Error loading configuration file"),
                //         String::from("Try launching the application again"),
                //     )),
                // }
            }
            WindowAsyncHandlerMsg::SetConfig(cfg) => {
                match editconfig(cfg) {
                    Ok(_) => sender.output(AppMsg::TryLoad),
                    Err(_) => sender.output(AppMsg::LoadError(
                        String::from("Error loading configuration file"),
                        String::from("Try launching the application again"),
                    )),
                };
            }
        }
    }
}

// fn configvalues() -> Result<(String, Option<String>), Box<dyn Error>> {
//     let path = checkconfig()?;
//     let config = readconfig(format!("{}/config.json", path))?;
//     Ok(config)
// }

pub struct LoadErrorModel {
    hidden: bool,
    msg: String,
    msg2: String,
}

#[derive(Debug)]
pub enum LoadErrorMsg {
    Show(String, String),
    Retry,
    Close,
    Preferences,
}

#[relm4::component(pub)]
impl SimpleComponent for LoadErrorModel {
    type InitParams = gtk::Window;
    type Input = LoadErrorMsg;
    type Output = AppMsg;
    type Widgets = LoadErrorWidgets;

    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            #[watch]
            set_text: Some(&model.msg),
            #[watch]
            set_secondary_text: Some(&model.msg2),
            set_use_markup: true,
            set_secondary_use_markup: true,
            add_button: ("Retry", gtk::ResponseType::Accept),
            add_button: ("Preferences", gtk::ResponseType::Help),
            add_button: ("Quit", gtk::ResponseType::Close),
            connect_response[sender] => move |_, resp| {
                sender.input(match resp {
                    gtk::ResponseType::Accept => LoadErrorMsg::Retry,
                    gtk::ResponseType::Close => LoadErrorMsg::Close,
                    gtk::ResponseType::Help => LoadErrorMsg::Preferences,
                    _ => unreachable!(),
                });
            },
        }
    }

    fn init(
        parent_window: Self::InitParams,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LoadErrorModel {
            hidden: true,
            msg: String::default(),
            msg2: String::default(),
        };
        let widgets = view_output!();
        let accept_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        accept_widget.add_css_class("warning");
        let pref_widget = widgets
            .dialog
            .widget_for_response(gtk::ResponseType::Help)
            .expect("No button for help response set");
        pref_widget.add_css_class("suggested-action");
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            LoadErrorMsg::Show(s, s2) => {
                self.hidden = false;
                self.msg = s;
                self.msg2 = s2;
            }
            LoadErrorMsg::Retry => {
                self.hidden = true;
                sender.output(AppMsg::TryLoad)
            }
            LoadErrorMsg::Close => sender.output(AppMsg::Close),
            LoadErrorMsg::Preferences => {
                sender.output(AppMsg::ShowPrefMenuErr);
                self.hidden = true;
            },
        }
    }
}
