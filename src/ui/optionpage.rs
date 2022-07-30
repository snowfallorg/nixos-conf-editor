use std::convert::identity;

use super::savechecking::*;
use super::window::*;
use crate::parse::options::OptionData;
use adw::prelude::*;
use html2pango;
use log::*;
use pandoc;
use relm4::*;
use sourceview5::prelude::*;

#[derive(Debug)]
pub enum OptPageMsg {
    UpdateOption(
        Box<OptionData>,
        Vec<String>,
        Vec<String>,
        String,
        Vec<String>,
    ),
    UpdateConf(String),
    UpdateConfMod(String),
    ResetConf,
    ClearConf,
    SaveConf,
    DoneSaving(bool, String),
    SetScheme(String),
}

#[tracker::track]
pub struct OptPageModel {
    pub opt: Vec<String>,
    pub refopt: Vec<String>,
    pub data: OptionData,
    pub conf: String,
    pub modifiedconf: String,
    alloptions: Vec<String>,
    scheme: Option<sourceview5::StyleScheme>,
    saving: bool,
    resettracker: u8,
    valuetracker: u8,
    #[tracker::no_eq]
    async_handler: WorkerController<SaveAsyncHandler>,
}

#[relm4::component(pub)]
impl SimpleComponent for OptPageModel {
    type InitParams = ();
    type Input = OptPageMsg;
    type Output = AppMsg;
    type Widgets = OptPageWidgets;

    view! {
        optwindow = gtk::ScrolledWindow {
            adw::Clamp {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 15,
                    set_spacing: 15,
                    set_vexpand: true,
                    add_css_class: "labels",
                    gtk::Label {
                        set_margin_top: 5,
                        set_margin_bottom: 5,
                        set_halign: gtk::Align::Start,
                        add_css_class: "title-1",
                        #[watch]
                        set_label: &model.opt.join(".")
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Description",
                            }
                        },
                        #[name(desc)]
                        gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            add_css_class: "body",
                            #[track(model.changed(OptPageModel::data()))]
                            set_markup: {
                                let x = format!("<article xmlns=\"http://docbook.org/ns/docbook\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" version=\"5.0\" xml:lang=\"en\"><para>\n{}\n</para></article>", model.data.description.trim());
                                let mut pandoc = pandoc::new();
                                pandoc.set_input(pandoc::InputKind::Pipe(x));
                                pandoc.set_output(pandoc::OutputKind::Pipe);
                                pandoc.set_input_format(pandoc::InputFormat::DocBook, vec![]);
                                pandoc.set_output_format(pandoc::OutputFormat::Html, vec![]);
                                let out = pandoc.execute().unwrap();
                                let y = match out {
                                    pandoc::PandocOutput::ToBuffer(s) => s,
                                    _ => "".to_string(),
                                };
                                let mut pango = html2pango::markup_html(&y.replace('\n', " \n")).unwrap_or(y).replace("• \n", "• ").trim().to_string();
                                while pango.ends_with('\n') {
                                    pango.pop();
                                }
                                pango.strip_prefix('\n').unwrap_or(&pango).to_string().as_str()
                            },
                        },
                    },


                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        #[name(type_header_box)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            #[name(type_header)]
                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Type",
                            }
                        },
                        gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            add_css_class: "body",
                            #[watch]
                            set_label: &model.data.op_type,
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        #[watch]
                        set_visible: model.data.default.is_some(),
                        #[name(default_box)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            #[name(default_header)]
                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Default",
                            }
                        },
                        gtk::Frame {
                            add_css_class: "code",
                            sourceview5::View {
                                set_editable: false,
                                set_monospace: true,
                                set_cursor_visible: false,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                #[wrap(Some)]
                                set_buffer: defaultbuf = &sourceview5::Buffer {
                                    #[track(model.changed(OptPageModel::scheme()))]
                                    set_style_scheme: model.scheme.as_ref(),
                                    #[watch]
                                    set_text: {
                                        let x = model.data.default.as_ref().unwrap_or(&serde_json::Value::Null);
                                        &match x {
                                            serde_json::Value::Object(o) => match o.get("text") {
                                                Some(serde_json::Value::String(s)) => {
                                                    if o.get("_type").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or("").eq("literalExpression") {
                                                        s.strip_suffix('\n').unwrap_or(s).to_string()
                                                    } else {
                                                        serde_json::to_string_pretty(x).unwrap_or_default()
                                                    }
                                                },
                                                    _ => serde_json::to_string_pretty(x).unwrap_or_default(),
                                            },
                                            _ => serde_json::to_string_pretty(x).unwrap_or_default(),
                                        }
                                    },
                                }
                            },
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        #[watch]
                        set_visible: model.data.example.is_some(),
                        #[name(example_box)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            #[name(example_header)]
                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                add_css_class: "h4",
                                set_label: "Example",
                            }
                        },
                        gtk::Frame {
                            add_css_class: "code",
                            sourceview5::View {
                                set_editable: false,
                                set_monospace: true,
                                set_cursor_visible: false,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                #[wrap(Some)]
                                set_buffer: exbuf = &sourceview5::Buffer {
                                    #[track(model.changed(OptPageModel::scheme()))]
                                    set_style_scheme: model.scheme.as_ref(),
                                    #[watch]
                                    set_text: {
                                        let x = model.data.example.as_ref().unwrap_or(&serde_json::Value::Null);
                                        &match x {
                                            serde_json::Value::Object(o) => match o.get("text") {
                                                Some(serde_json::Value::String(s)) => {
                                                    if o.get("_type").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or("").eq("literalExpression") {
                                                        s.strip_suffix('\n').unwrap_or(s).to_string()
                                                    } else {
                                                        serde_json::to_string_pretty(x).unwrap_or_default()
                                                    }
                                                },
                                                    _ => serde_json::to_string_pretty(x).unwrap_or_default(),
                                            },
                                            _ => serde_json::to_string_pretty(x).unwrap_or_default(),
                                        }
                                    },
                                }
                            },
                        },
                    },
                    gtk::Separator {
                        set_opacity: 0.0,
                        set_margin_top: 5,
                    },
                    gtk::Box {
                        #[watch]
                        set_visible: valuestack.is_child_visible(),
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        #[name(simplevalue_box)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Value",
                            }
                        },
                        #[name(valuestack)]
                        gtk::Stack {
                            #[name(number)]
                            gtk::SpinButton {
                                set_halign: gtk::Align::Start,
                                set_adjustment: &gtk::Adjustment::new(0.0, f64::MIN, f64::MAX, 1.0, 5.0, 0.0),
                                set_climb_rate: 1.0,
                                set_digits: 0,
                                connect_value_changed[sender] => move |x| {
                                    if x.is_sensitive() {
                                        sender.input(OptPageMsg::UpdateConfMod(x.value().to_string()))
                                    }
                                },
                            },
                            #[name(stringentry)]
                            gtk::Entry {
                                set_halign: gtk::Align::Start,
                                connect_changed[sender] => move |x| {
                                    if x.is_sensitive() {
                                        sender.input(OptPageMsg::UpdateConfMod(format!("\"{}\"", x.text())));
                                    }
                                },
                            },
                            #[name(truefalse)]
                            gtk::Box {
                                add_css_class: "linked",
                                set_orientation: gtk::Orientation::Horizontal,
                                #[name(truebtn)]
                                gtk::ToggleButton {
                                    set_label: "True",
                                    connect_toggled[sender] => move |x| {
                                        if x.is_active() {
                                            sender.input(OptPageMsg::UpdateConfMod(String::from("true")))
                                        }
                                    }
                                },
                                #[name(falsebtn)]
                                gtk::ToggleButton {
                                    set_label: "False",
                                    set_group: Some(&truebtn),
                                    connect_toggled[sender] => move |x| {
                                        if x.is_active() {
                                            sender.input(OptPageMsg::UpdateConfMod(String::from("false")))
                                        }
                                    }
                                },
                                // #[name(nullbtn)]
                                // gtk::ToggleButton {
                                //     set_label: "null",
                                //     set_group: Some(&truebtn),
                                // }
                            },
                        }
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        #[name(value_box)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Attribute Value",
                            }
                        },
                        gtk::Frame {
                            add_css_class: "code",
                            sourceview5::View {
                                set_background_pattern: sourceview5::BackgroundPatternType::Grid,
                                set_height_request: 100,
                                set_editable: true,
                                set_monospace: true,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                #[wrap(Some)]
                                set_buffer: valuebuf = &sourceview5::Buffer {
                                    #[track(model.changed(OptPageModel::scheme()))]
                                    set_style_scheme: model.scheme.as_ref(),
                                    #[track(model.changed(OptPageModel::opt()))]
                                    set_text: {
                                        debug!("opt changing valuebuf to {:?}", model.conf);
                                        &model.conf
                                    },
                                    #[track(model.changed(OptPageModel::conf()))]
                                    set_text: {
                                        debug!("conf changing valuebuf to {:?}", model.conf);
                                        &model.conf
                                    },
                                    #[track(model.changed(OptPageModel::valuetracker()))]
                                    set_text: {
                                        debug!("valuetracker changing valuebuf to {:?}", model.modifiedconf);
                                        &model.modifiedconf
                                    },
                                    connect_changed[sender] => move |x| {
                                        let (start, end) = x.bounds();
                                        debug!("valuebuf changed to {:?}", x.text(&start, &end, true));
                                        let text = x.text(&start, &end, true).to_string();
                                        sender.input(OptPageMsg::UpdateConf(text))
                                    }
                                }
                            },
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            gtk::Button {
                                set_label: "Reset",
                                #[watch]
                                set_sensitive: model.conf != model.modifiedconf,
                                connect_clicked[sender] => move |_| {
                                    sender.input(OptPageMsg::ResetConf)
                                }
                            },
                            gtk::Button {
                                set_label: "Clear",
                                #[watch]
                                set_sensitive: !model.modifiedconf.is_empty(),
                                connect_clicked[sender] => move |_| {
                                    sender.input(OptPageMsg::ClearConf)
                                }
                            },
                            #[name(savestack)]
                            gtk::Stack {
                                set_halign: gtk::Align::End,
                                set_hexpand: true,
                                #[name(savebtn)]
                                gtk::Button {
                                    set_label: "Save",
                                    add_css_class: "suggested-action",
                                    #[watch]
                                    set_sensitive: model.conf != model.modifiedconf,
                                    connect_clicked[sender] => move |_| {
                                        sender.input(OptPageMsg::SaveConf)
                                    },
                                },
                                #[name(spinner)]
                                gtk::Spinner {
                                    #[watch]
                                    set_spinning: model.saving,
                                },
                            },
                        }
                    }
                }
            }
        }
    }

    fn pre_view() {
        info!("pre_view");
        let set_val = || {
            debug!("SET VAL");
            if let Some(x) = valuestack.visible_child() {
                let val = model.conf.as_str();
                if x.eq(truefalse) {
                    if val == "true" {
                        truebtn.set_active(true);
                        falsebtn.set_active(false);
                    } else if val == "false" {
                        truebtn.set_active(false);
                        falsebtn.set_active(true);
                    } else {
                        truebtn.set_active(false);
                        falsebtn.set_active(false);
                    }
                } else if x.eq(number) {
                    number.set_sensitive(false);
                    if let Ok(x) = val.parse::<f64>() {
                        number.set_value(x);
                    } else {
                        number.set_value(0.0);
                    }
                    number.set_sensitive(true);
                } else if x.eq(stringentry) {
                    if let Some(x) = val.chars().next() {
                        if let Some(y) = val.chars().last() {
                            if x == '"' && y == '"' {
                                if let Some(v) = val.get(1..val.len() - 1) {
                                    stringentry.set_sensitive(false);
                                    stringentry.set_text(v);
                                    stringentry.set_sensitive(true);
                                    return;
                                }
                            }
                        }
                    }
                    stringentry.set_sensitive(false);
                    stringentry.set_text("");
                    stringentry.set_sensitive(true);
                } else {
                    warn!("Unhandled valuestack child {:?}", x);
                }
            } else {
                info!("No simple value widget for type '{}'", model.data.op_type);
            }
        };

        if model.changed(OptPageModel::opt()) {
            let optype = model.data.op_type.as_str();
            valuestack.set_child_visible(true);
            match optype {
                "boolean" | "null or boolean" => valuestack.set_visible_child(truefalse),
                "signed integer" | "null or signed integer" => valuestack.set_visible_child(number),
                "string" | "null or string" | "string, not containing newlines or colons" => {
                    valuestack.set_visible_child(stringentry)
                }
                _ => valuestack.set_child_visible(false),
            }
            if valuestack.is_child_visible() {
                set_val();
            }
        }
        if model.changed(OptPageModel::resettracker()) {
            // Reset button is pressed
            set_val();
        }
        if model.saving {
            savestack.set_visible_child(spinner)
        } else {
            savestack.set_visible_child(savebtn)
        }
    }

    fn init(
        _parent_window: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let async_handler = SaveAsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);
        let model = OptPageModel {
            opt: vec![],    //parent_window.position.clone(),
            refopt: vec![], //parent_window.refposition.clone(),
            data: OptionData::default(),
            conf: String::new(),
            modifiedconf: String::new(),
            saving: false,
            alloptions: vec![], //parent_window.data.keys().map(|x| x.to_string()).collect::<Vec<String>>(),
            scheme: None,
            resettracker: 0,
            valuetracker: 0,
            async_handler,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        self.reset();
        match msg {
            OptPageMsg::UpdateOption(data, opt, refopt, conf, alloptions) => {
                info!("OptPageMsg::UpdateOption");
                self.update_conf(|x| x.clear());
                self.update_modifiedconf(|x| x.clear());
                self.set_data(*data);
                self.update_opt(|o| *o = opt.to_vec());
                self.set_refopt(refopt);
                self.set_conf(conf.clone());
                self.set_modifiedconf(conf);
                self.set_alloptions(alloptions);
            }
            OptPageMsg::UpdateConf(conf) => {
                info!("OptPageMsg::UpdateConf");
                if conf != self.modifiedconf {
                    self.set_modifiedconf(conf);
                }
            }
            OptPageMsg::UpdateConfMod(conf) => {
                info!("OptPageMsg::UpdateConfMod");
                if conf != self.modifiedconf {
                    self.set_modifiedconf(conf);
                    self.update_valuetracker(|_| ()); // Simulate change to conf
                }
            }
            OptPageMsg::ResetConf => {
                info!("OptPageMsg::ResetConf");
                let conf = self.conf.clone();
                self.set_modifiedconf(conf);
                self.update_valuetracker(|_| ()); // Simulate change to conf
                self.update_resettracker(|_| ()); // Simulate reset
            }
            OptPageMsg::ClearConf => {
                info!("OptPageMsg::ClearConf");
                self.set_modifiedconf(String::default());
                self.update_valuetracker(|_| ()); // Simulate change to conf
            }
            OptPageMsg::SaveConf => {
                info!("OptPageMsg::SaveConf");
                let opt = self.opt.join(".");
                let refopt = self.refopt.join(".");
                let mut conf = self.modifiedconf.clone();
                while conf.ends_with('\n') || conf.ends_with(' ') {
                    conf.pop();
                }
                self.set_modifiedconf(conf.clone());
                if conf.is_empty() {
                    sender.input(OptPageMsg::DoneSaving(true, "true\n".to_string()));
                } else {
                    self.set_saving(true);
                    sender.output(AppMsg::SetBusy(true));
                    self.async_handler.emit(SaveAsyncHandlerMsg::SaveCheck(
                        opt,
                        refopt,
                        conf,
                        self.alloptions.to_vec(),
                    ));
                }
            }
            OptPageMsg::DoneSaving(save, message) => {
                info!("OptPageMsg::DoneSaving");
                if save {
                    if message.eq("true\n") {
                        //Save
                        self.set_conf(self.modifiedconf.clone());
                        sender.output(AppMsg::EditOpt(
                            self.opt.join("."),
                            self.modifiedconf.clone(),
                        ));
                        self.update_resettracker(|_| ()); // Simulate reset
                    } else {
                        //Type mismatch
                        let e =
                            format!("{} is not of type {}", self.modifiedconf, self.data.op_type);
                        sender.output(AppMsg::SaveError(e));
                    }
                } else {
                    //Error
                    sender.output(AppMsg::SaveError(message));
                }

                self.set_saving(false);
                sender.output(AppMsg::SetBusy(false));
            }
            OptPageMsg::SetScheme(scheme) => {
                info!("OptPageMsg::SetScheme");
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
        }
    }
}
