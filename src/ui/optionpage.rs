use crate::parse::options::OptionData;
use adw::prelude::*;
use relm4::*;
use sourceview5::prelude::*;
use pandoc;
use html2pango;
use super::window::*;
use super::savechecking::*;

pub enum OptPageMsg {
    UpdateOption(OptionData, Vec<String>, Vec<String>, String),
    UpdateConf(String),
    ResetConf,
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
    scheme: Option<sourceview5::StyleScheme>,
    saving: bool,
}

impl Model for OptPageModel {
    type Msg = OptPageMsg;
    type Widgets = OptPageWidgets;
    type Components = OptPageComponents;
}

#[derive(relm4::Components)]
pub struct OptPageComponents {
    async_handler: RelmMsgHandler<SaveAsyncHandler, OptPageModel>,
    //saveerror: RelmComponent<SaveErrorModel, OptPageModel>,
}

#[relm4::async_trait]
impl ComponentUpdate<AppModel> for OptPageModel {
    fn init_model(parent_model: &AppModel) -> Self {
        OptPageModel {
            opt: parent_model.position.clone(),
            refopt: parent_model.refposition.clone(),
            data: OptionData::default(),
            conf: String::new(),
            modifiedconf: String::new(),
            saving: false,
            scheme: None,
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: OptPageMsg,
        components: &OptPageComponents,
        sender: Sender<OptPageMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            OptPageMsg::UpdateOption(data, opt, refopt, conf) => {
                self.update_conf(|x| x.clear());
                self.update_modifiedconf(|x| x.clear());
                self.set_data(data);
                self.set_opt(opt);
                self.set_refopt(refopt);
                self.set_conf(conf.clone());
                self.set_modifiedconf(conf);
            }
            OptPageMsg::UpdateConf(conf) => {
                if conf != self.modifiedconf {
                    self.set_modifiedconf(conf);
                }
            }
            OptPageMsg::ResetConf => {
                let conf = self.conf.clone();
                self.conf.clear();
                self.set_conf(conf.clone());
                self.set_modifiedconf(conf);
                self.update_conf(|_| ()); // Simulate change to conf
            }
            OptPageMsg::SaveConf => {
                let opt = self.opt.join(".");
                let refopt = self.refopt.join(".");
                let mut conf = self.modifiedconf.clone();
                while conf.ends_with('\n') || conf.ends_with(' ') {
                    conf.pop();
                }
                self.set_modifiedconf(conf.clone());
                if conf.is_empty() {
                    send!(sender, OptPageMsg::DoneSaving(true, "true\n".to_string()));
                } else {
                    self.set_saving(true);
                    parent_sender.send(AppMsg::SetBusy(true)).unwrap();
                    components.async_handler.sender().blocking_send(SaveAsyncHandlerMsg::SaveCheck(opt, refopt, conf)).expect("Could not send message to async handler");
                }
            }
            OptPageMsg::DoneSaving(save, message) => {
                if save {
                    if message.eq("true\n") {
                        //Save
                        self.set_conf(self.modifiedconf.clone());
                        parent_sender.send(AppMsg::EditOpt(self.opt.join("."), self.modifiedconf.clone())).unwrap();
                        
                    } else {
                        //Type mismatch
                        let e = format!("{} is not of type {}", self.modifiedconf, self.data.op_type);
                        parent_sender.send(AppMsg::SaveError(e)).unwrap();
                    }
                } else {
                    //Error
                    parent_sender.send(AppMsg::SaveError(message)).unwrap();
                }
                
                self.set_saving(false);
                parent_sender.send(AppMsg::SetBusy(false)).unwrap();
            }
            OptPageMsg::SetScheme(scheme) => {
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(&scheme));
            }
        }
    }
}

#[relm4::widget(pub)]
impl Widgets<OptPageModel, AppModel> for OptPageWidgets {
    view! {
        optwindow = gtk::ScrolledWindow {
            set_child = Some(&adw::Clamp) {
                set_child = Some(&gtk::Box) {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 15,
                    set_spacing: 15,
                    set_vexpand: true,
                    add_css_class: "labels",
                    append = &gtk::Label {
                        set_margin_top: 5,
                        set_margin_bottom: 5,
                        set_halign: gtk::Align::Start,
                        add_css_class: "title-1",
                        set_label: watch!(&model.opt.join("."))
                    },

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append: desc_header_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append: desc_header = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Description",
                            }
                        },
                        append = &gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            add_css_class: "body",
                            set_use_markup: true,
                            set_label: watch!({
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
                            }),
                        },
                    },


                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append: type_header_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append: type_header = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Type",
                            }
                        },
                        append = &gtk::Label {
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            add_css_class: "body",
                            set_label: watch!(&model.data.op_type),
                        },
                    },

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        set_visible: watch!(model.data.default.is_some()),
                        append: default_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append: default_header = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Default",
                            }
                        },
                        append = &gtk::Frame {
                            add_css_class: "code",
                            set_child = Some(&sourceview5::View) {
                                set_editable: false,
                                set_monospace: true,
                                set_cursor_visible: false,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                set_buffer: defaultbuf = Some(&sourceview5::Buffer) {
                                    set_style_scheme: track!(model.changed(OptPageModel::scheme()), model.scheme.as_ref()),
                                    set_text: watch!({
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
                                    }),
                                }
                            },
                        },
                    },

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        set_visible: watch!(model.data.example.is_some()),
                        append: example_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append: example_header = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                add_css_class: "h4",
                                set_label: "Example",
                            }
                        },
                        append = &gtk::Frame {
                            add_css_class: "code",
                            set_child = Some(&sourceview5::View) {
                                set_editable: false,
                                set_monospace: true,
                                set_cursor_visible: false,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                set_buffer: exbuf = Some(&sourceview5::Buffer) {
                                    set_style_scheme: track!(model.changed(OptPageModel::scheme()), model.scheme.as_ref()),
                                    set_text: watch!({
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
                                    }),
                                }
                            },
                        },
                    },

                    append = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        append: value_box = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            add_css_class: "header",
                            add_css_class: "single-line",
                            append = &gtk::Label {
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                                set_label: "Attribute Value",
                            }
                        },
                        append = &gtk::Frame {
                            add_css_class: "code",
                            set_child = Some(&sourceview5::View) {
                                set_background_pattern: sourceview5::BackgroundPatternType::Grid,
                                set_height_request: 100,
                                set_editable: true,
                                set_monospace: true,
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                set_buffer: valuebuf = Some(&sourceview5::Buffer) {
                                    set_style_scheme: track!(model.changed(OptPageModel::scheme()), model.scheme.as_ref()),
                                    set_text: track!(model.changed(OptPageModel::conf()) || model.changed(OptPageModel::opt()), &model.conf),
                                    connect_changed(sender) => move |x| {
                                        let (start, end) = x.bounds();
                                        let text = x.text(&start, &end, true).to_string();
                                        send!(sender, OptPageMsg::UpdateConf(text))
                                    }
                                }
                            },
                        },
                        append = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            append = &gtk::Button {
                                set_label: "Reset",
                                set_sensitive: watch!(model.conf != model.modifiedconf),
                                connect_clicked(sender) => move |_| {
                                    send!(sender, OptPageMsg::ResetConf)
                                }
                            },
                            append: savestack = &gtk::Stack {
                                set_halign: gtk::Align::End,
                                set_hexpand: true,
                                add_child: savebtn = &gtk::Button {
                                    set_label: "Save",
                                    add_css_class: "suggested-action",
                                    set_sensitive: watch!(model.conf != model.modifiedconf),
                                    connect_clicked(sender) => move |_| {
                                        send!(sender, OptPageMsg::SaveConf)
                                    },
                                },
                                add_child: spinner = &gtk::Spinner {
                                    set_spinning: watch!(model.saving),
                                },
                            },
                        }
                    }
                }
            }
        }
    }

    fn pre_view() {
        if model.saving {
            self.savestack.set_visible_child(&self.spinner)
        } else {
            self.savestack.set_visible_child(&self.savebtn)
        }
    }
}
