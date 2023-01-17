use super::window::*;
use adw::prelude::*;
use relm4::{factory::*, *};
use std::collections::HashMap;

pub struct SearchEntryModel {
    hidden: bool,
    position: Vec<String>,
    data: FactoryVecDeque<SearchEntryOption>,
    nameopts: FactoryVecDeque<SearchNameEntryOption>,
    customopt: Vec<String>,
}

#[derive(Debug)]
pub enum SearchEntryMsg {
    Show(Vec<String>, Vec<String>),
    Close,
    Save(Option<Vec<String>>),
    SetName(String, usize),
}

#[relm4::component(pub)]
impl SimpleComponent for SearchEntryModel {
    type Init = gtk::Window;
    type Input = SearchEntryMsg;
    type Output = AppMsg;
    type Widgets = SearchEntryWidgets;

    view! {
        window = gtk::Dialog {
            set_default_height: -1,
            #[wrap(Some)]
            set_titlebar = &adw::HeaderBar {
                add_css_class: "flat"
            },
            set_default_width: 500,
            set_resizable: false,
            set_transient_for: Some(&parent_window),
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            connect_close_request[sender] => move |_| {
                sender.input(SearchEntryMsg::Close);
                gtk::Inhibit(true)
            },
            connect_visible_notify => move |x| {
                if x.get_visible() {
                    x.grab_focus();
                }
            },
            #[name(main_box)]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                adw::Clamp {
                    gtk::Box {
                        set_margin_start: 15,
                        set_margin_end: 15,
                        set_spacing: 15,
                        set_orientation: gtk::Orientation::Vertical,
                        adw::PreferencesGroup {
                            #[local_ref]
                            add = datalistbox -> gtk::ListBox {
                                add_css_class: "boxed-list",
                            },
                            #[watch]
                            set_visible: !model.data.is_empty(),
                        },
                        adw::PreferencesGroup {
                            #[local_ref]
                            add = nameoptslistbox -> gtk::ListBox {
                                set_margin_bottom: 15,
                                add_css_class: "boxed-list",
                                append = &adw::ActionRow {
                                    #[watch]
                                    set_title: &model.customopt.join("."),
                                    #[watch]
                                    set_sensitive: !model.customopt.contains(&String::from("&lt;name&gt;")),
                                    set_selectable: false,
                                    set_activatable: true,
                                    add_suffix = &gtk::Image {
                                        set_icon_name: Some("list-add-symbolic"),
                                        add_css_class: "accent",
                                    },
                                    connect_activated[sender] => move |_| {
                                        sender.input(SearchEntryMsg::Save(None));
                                    }
                                }
                            }
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
        let model = SearchEntryModel {
            hidden: true,
            position: Vec::default(),
            data: FactoryVecDeque::new(gtk::ListBox::new(), sender.input_sender()),
            nameopts: FactoryVecDeque::new(gtk::ListBox::new(), sender.input_sender()),
            customopt: Vec::default(),
        };
        let datalistbox = model.data.widget();
        let nameoptslistbox = model.nameopts.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        let mut data_guard = self.data.guard();
        let mut nameopts_guard = self.nameopts.guard();
        match msg {
            SearchEntryMsg::Show(pos, optdata) => {
                data_guard.clear();
                nameopts_guard.clear();
                for v in optdata {
                    data_guard.push_back(v);
                }
                for i in 0..pos.len() {
                    if pos[i] == "<name>" {
                        let mut op = pos.to_vec();
                        op[i] = "<b>&lt;name&gt;</b>".to_string();
                        nameopts_guard
                            .push_back((op.join(".").replace("<name>", "&lt;name&gt;"), i));
                    }
                }
                self.position = pos.clone();
                self.customopt = pos
                    .iter()
                    .map(|x| x.replace("<name>", "&lt;name&gt;"))
                    .collect();
                self.hidden = false;
            }
            SearchEntryMsg::Close => {
                self.hidden = true;
                data_guard.clear();
            }
            SearchEntryMsg::Save(dest) => {
                self.hidden = true;
                let mut n: HashMap<usize, usize> = HashMap::new();
                if dest.is_none() {
                    for i in 0..self.position.len() {
                        if self.position[i] == "<name>" {
                            let mut existing = vec![];
                            for i in 0..data_guard.len() {
                                let dvec = &data_guard.get(i).unwrap().value;
                                if !existing.contains(&dvec[i].to_string())
                                    && dvec[..i] == self.customopt[..i]
                                {
                                    existing.push(dvec[i].to_string());
                                }
                            }
                            if !existing.contains(&self.customopt[i]) {
                                let _ = sender.output(AppMsg::AddNameAttr(
                                    Some(self.customopt[..i].join(".")),
                                    self.customopt[i].clone(),
                                ));
                            }
                        } else if self.position[i] == "*" {
                            let mut existing = vec![];
                            for i in 0..data_guard.len() {
                                let dvec = &data_guard.get(i).unwrap().value;
                                if dvec[..i] == self.customopt[..i] {
                                    if let Some(v) = dvec.get(i) {
                                        if let Ok(x) = v.parse::<usize>() {
                                            if !existing.contains(&x) {
                                                existing.push(x);
                                            }
                                        }
                                    }
                                }
                            }
                            existing.sort_unstable();
                            let num = if let Some(x) = existing.last() {
                                *x + 1
                            } else {
                                0
                            };
                            n.insert(i, num);
                            let _ = sender.output(AppMsg::AddStar(self.customopt[..i].join(".")));
                        }
                    }
                    for (k, v) in n {
                        self.customopt[k] = v.to_string();
                    }
                }
                data_guard.clear();
                let _ = sender.output(AppMsg::OpenSearchOption(
                    if let Some(x) = dest {
                        x
                    } else {
                        self.customopt.to_vec()
                    },
                    self.position.to_vec(),
                ));
            }
            SearchEntryMsg::SetName(v, i) => {
                if self.position.get(i).is_some() {
                    if v.is_empty() {
                        self.customopt[i] = String::from("&lt;name&gt;");
                    } else {
                        self.customopt[i] = v;
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct SearchEntryOption {
    value: Vec<String>,
}

#[derive(Debug)]
enum SearchEntryOptionOutput {
    Save(Vec<String>),
}

#[relm4::factory]
impl FactoryComponent for SearchEntryOption {
    type Init = String;
    type Input = ();
    type Output = SearchEntryOptionOutput;
    type Widgets = CounterWidgets;
    type ParentWidget = gtk::ListBox;
    type ParentInput = SearchEntryMsg;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.value.join("."),
            set_selectable: false,
            set_activatable: true,
            connect_activated[sender, value = self.value.clone()] => move |_| {
                sender.output(SearchEntryOptionOutput::Save(value.to_vec()));
            }
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let v = value
            .split('.')
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        Self { value: v }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<SearchEntryMsg> {
        Some(match output {
            SearchEntryOptionOutput::Save(v) => SearchEntryMsg::Save(Some(v)),
        })
    }
}

#[derive(Debug, PartialEq)]
struct SearchNameEntryOption {
    value: String,
    index: usize,
}

#[derive(Debug)]
enum SearchNameEntryOptionOutput {
    SetName(String, usize),
}

#[relm4::factory]
impl FactoryComponent for SearchNameEntryOption {
    type Init = (String, usize);
    type Input = ();
    type Output = SearchNameEntryOptionOutput;
    type Widgets = SearchNameWidgets;
    type ParentWidget = gtk::ListBox;
    type ParentInput = SearchEntryMsg;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.value,
            add_suffix = &gtk::Entry {
                set_valign: gtk::Align::Center,
                set_placeholder_text: Some("<name>"),
                set_buffer = &gtk::EntryBuffer {
                    connect_text_notify[sender, index = self.index] => move |x| {
                        sender.output(SearchNameEntryOptionOutput::SetName(x.text(), index));
                    }
                }
            },
            set_selectable: false,
            set_activatable: false,
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            value: value.0,
            index: value.1,
        }
    }

    fn output_to_parent_input(output: Self::Output) -> Option<SearchEntryMsg> {
        Some(match output {
            SearchNameEntryOptionOutput::SetName(x, i) => SearchEntryMsg::SetName(x, i),
        })
    }
}
