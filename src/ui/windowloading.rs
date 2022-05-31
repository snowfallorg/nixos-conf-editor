use std::error::Error;
use adw::prelude::*;
use relm4::{
    send, ComponentUpdate, MessageHandler, Model, Sender, Widgets,
};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};
use crate::parse::cache::checkcache;
use crate::parse::config::parseconfig;
use crate::parse::options::read;
use crate::parse::preferences::{checkconfig, readconfig, configexists, createconfig};
use super::window::{AppModel, AppMsg, LoadValues};

pub struct WindowAsyncHandler {
    _rt: Runtime,
    pub sender: TokioSender<WindowAsyncHandlerMsg>,
}

#[derive(Debug)]
pub enum WindowAsyncHandlerMsg {
    RunWindow(String),
    GetConfigPath,
    SetConfig(String, Option<String>),
}

impl MessageHandler<AppModel> for WindowAsyncHandler {
    type Msg = WindowAsyncHandlerMsg;
    type Sender = TokioSender<WindowAsyncHandlerMsg>;

    fn init(_parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        let (sender, mut rx) = channel::<WindowAsyncHandlerMsg>(10);

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
                        WindowAsyncHandlerMsg::RunWindow(path) => {
                            match checkcache() {
                                Ok(_) => {},
                                Err(_) => {
                                    send!(parent_sender, AppMsg::LoadError(String::from("Could not load cache"), String::from("Try connecting to the internet or launching the application again")));
                                }
                            }

                            match setupfiles(&path) {
                                Ok(x) => {
                                    send!(parent_sender, AppMsg::InitialLoad(x))
                                }
                                Err(_) => {
                                    println!("GOT ERROR");
                                    send!(parent_sender, AppMsg::LoadError(String::from("Error loading configuration file"), format!("<tt>{}</tt> may be an invalid configuration file", path)))
                                }
                            }
                        }
                        WindowAsyncHandlerMsg::GetConfigPath => {
                            if let Ok(false) = configexists() {
                                send!(parent_sender, AppMsg::Welcome);
                                return
                            }
                            match configvalues() {
                                Ok((x, y)) => {
                                    send!(parent_sender, AppMsg::SetConfPath(x, y))
                                }
                                Err(_) => {
                                    send!(parent_sender, AppMsg::LoadError(String::from("Error loading configuration file"), String::from("Try launching the application again")))
                                }
                            }
                        }
                        WindowAsyncHandlerMsg::SetConfig(path, flake) => {
                            match createconfig(path, flake) {
                                Ok(_) => {
                                    send!(parent_sender, AppMsg::TryLoad)
                                }
                                Err(_) => {
                                    send!(parent_sender, AppMsg::LoadError(String::from("Error loading configuration file"), String::from("Try launching the application again")))
                                }
                            };
                        }
                    }
                });
            }
        });

        WindowAsyncHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}

fn setupfiles(configpath: &str) -> Result<LoadValues, Box<dyn Error>> {
    let (data, tree) = read()?;
    let conf = parseconfig(configpath)?;
    Ok(LoadValues { data, tree, conf })
}

fn configvalues() -> Result<(String, Option<String>), Box<dyn Error>> {
    let path = checkconfig()?;
    let config = readconfig(format!("{}/config.json", path))?;
    Ok(config)
}


pub struct LoadErrorModel {
    hidden: bool,
    msg: String,
    msg2: String,
}

pub enum LoadErrorMsg {
    Show(String, String),
    Retry,
    Close,
    Preferences,
}

impl Model for LoadErrorModel {
    type Msg = LoadErrorMsg;
    type Widgets = LoadErrorWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for LoadErrorModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        LoadErrorModel {
            hidden: true,
            msg: String::default(),
            msg2: String::default(),
        }
    }

    fn update(
        &mut self,
        msg: LoadErrorMsg,
        _components: &(),
        _sender: Sender<LoadErrorMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            LoadErrorMsg::Show(s, s2) => {
                self.hidden = false;
                self.msg = s;
                self.msg2 = s2;
            },
            LoadErrorMsg::Retry => {
                self.hidden = true;
                send!(parent_sender, AppMsg::TryLoad)
            },
            LoadErrorMsg::Close => {
                send!(parent_sender, AppMsg::Close)
            },
            LoadErrorMsg::Preferences => {
                send!(parent_sender, AppMsg::ShowPrefMenu)
            },
        }
    }
}


#[relm4::widget(pub)]
impl Widgets<LoadErrorModel, AppModel> for LoadErrorWidgets {
    view! {
        dialog = gtk::MessageDialog {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            set_text: watch!(Some(&model.msg)),
            set_secondary_text: watch!(Some(&model.msg2)),
            set_use_markup: true,
            set_secondary_use_markup: true,
            add_button: args!("Retry", gtk::ResponseType::Accept),
            add_button: args!("Preferences", gtk::ResponseType::Help),
            add_button: args!("Quit", gtk::ResponseType::Close),
            connect_response(sender) => move |_, resp| {
                send!(sender, match resp {
                    gtk::ResponseType::Accept => LoadErrorMsg::Retry,
                    gtk::ResponseType::Close => LoadErrorMsg::Close,
                    gtk::ResponseType::Help => LoadErrorMsg::Preferences,
                    _ => unreachable!(),
                });
            },
        }
    }

    fn post_init() {
        let accept_widget = dialog
            .widget_for_response(gtk::ResponseType::Accept)
            .expect("No button for accept response set");
        accept_widget.add_css_class("warning");
        let pref_widget = dialog
            .widget_for_response(gtk::ResponseType::Help)
            .expect("No button for help response set");
        pref_widget.add_css_class("suggested-action");
    }

}
