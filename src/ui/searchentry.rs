use std::collections::HashMap;

use super::window::*;
use adw::prelude::*;
use relm4::{
    factory::{FactoryPrototype, FactoryVec},
    *,
};

#[tracker::track]
pub struct SearchEntryModel {
    hidden: bool,
    position: Vec<String>,
    #[tracker::no_eq]
    data: FactoryVec<SearchEntryOption>,
    #[tracker::no_eq]
    nameopts: FactoryVec<SearchNameEntryOption>,
    customopt: Vec<String>,
}

pub enum SearchEntryMsg {
    Show(Vec<String>, Vec<String>),
    Close,
    Save(Option<Vec<String>>),
    SetName(String, usize),
}

impl Model for SearchEntryModel {
    type Msg = SearchEntryMsg;
    type Widgets = SearchEntryWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for SearchEntryModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        SearchEntryModel {
            hidden: true,
            position: Vec::default(),
            data: FactoryVec::new(),
            nameopts: FactoryVec::new(),
            customopt: Vec::default(),
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: SearchEntryMsg,
        _components: &(),
        _sender: Sender<SearchEntryMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();
        match msg {
            SearchEntryMsg::Show(pos, optdata) => {
                self.data.clear();
                self.nameopts.clear();
                for v in optdata {
                    self.data.push(SearchEntryOption { value: v });
                }
                for i in 0..pos.len() {
                    if pos[i] == "<name>" {
                        let mut op = pos.to_vec();
                        op[i] = "<b>&lt;name&gt;</b>".to_string();
                        self.nameopts.push(SearchNameEntryOption {
                            value: op.join(".").replace("<name>", "&lt;name&gt;"),
                            index: i,
                        });
                    }
                }
                self.position = pos.clone();
                self.customopt = pos
                    .iter()
                    .map(|x| x.replace("<name>", "&lt;name&gt;"))
                    .collect();
                self.set_hidden(false);
            }
            SearchEntryMsg::Close => {
                self.set_hidden(true);
                self.data.clear();
            }
            SearchEntryMsg::Save(dest) => {
                self.set_hidden(true);
                let mut n: HashMap<usize, usize> = HashMap::new();
                if dest.is_none() {
                    for i in 0..self.position.len() {
                        if self.position[i] == "<name>" {
                            let mut existing = vec![];
                            for d in self.data.iter() {
                                let dvec = d.value.split('.').collect::<Vec<_>>();
                                if !existing.contains(&dvec[i].to_string())
                                    && dvec[..i] == self.customopt[..i]
                                {
                                    existing.push(dvec[i].to_string());
                                }
                            }
                            if !existing.contains(&self.customopt[i]) {
                                send!(
                                    parent_sender,
                                    AppMsg::AddNameAttr(
                                        Some(self.customopt[..i].join(".")),
                                        self.customopt[i].clone()
                                    )
                                );
                            }
                        } else if self.position[i] == "*" {
                            let mut existing = vec![];
                            for d in self.data.iter() {
                                let dvec = d.value.split('.').collect::<Vec<_>>();
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
                            send!(parent_sender, AppMsg::AddStar(self.customopt[..i].join(".")));
                        }
                    }
                    for (k, v) in n {
                        self.customopt[k] = v.to_string();
                    }
                }
                self.data.clear();
                send!(
                    parent_sender,
                    AppMsg::OpenSearchOption(
                        if let Some(x) = dest {
                            x
                        } else {
                            self.customopt.to_vec()
                        },
                        self.position.to_vec()
                    )
                );
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

#[relm4::widget(pub)]
impl Widgets<SearchEntryModel, AppModel> for SearchEntryWidgets {
    view! {
        window = gtk::Dialog {
            set_default_height: -1,
            set_titlebar = Some(&adw::HeaderBar) {
                add_css_class: "flat"
            },
            set_default_width: 500,
            set_resizable: false,
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_modal: true,
            set_visible: watch!(!model.hidden),
            connect_close_request(sender) => move |_| {
                send!(sender, SearchEntryMsg::Close);
                gtk::Inhibit(true)
            },
            connect_visible_notify => move |x| {
                if x.get_visible() {
                    x.grab_focus();
                }
            },
            set_child: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,
                append = &adw::Clamp {
                    set_child = Some(&gtk::Box) {
                        set_margin_start: 15,
                        set_margin_end: 15,
                        set_spacing: 15,
                        set_orientation: gtk::Orientation::Vertical,
                        append = &adw::PreferencesGroup {
                            add = &gtk::ListBox {
                                add_css_class: "boxed-list",
                                factory!(model.data)
                            },
                            set_visible: watch!(!model.data.is_empty()),
                        },
                        append = &adw::PreferencesGroup {
                            add = &gtk::ListBox {
                                set_margin_bottom: 15,
                                add_css_class: "boxed-list",
                                factory!(model.nameopts),
                                append = &adw::ActionRow {
                                    set_title: watch!(&model.customopt.join(".")),
                                    set_sensitive: watch!(!model.customopt.contains(&String::from("&lt;name&gt;"))),
                                    set_selectable: false,
                                    set_activatable: true,
                                    add_suffix = &gtk::Image {
                                        set_icon_name: Some("list-add-symbolic"),
                                        add_css_class: "accent",
                                    },
                                    connect_activated(sender) => move |_| {
                                        send!(sender, SearchEntryMsg::Save(None));
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct SearchEntryOption {
    value: String,
}

#[relm4::factory_prototype]
impl FactoryPrototype for SearchEntryOption {
    type Factory = FactoryVec<Self>;
    type Widgets = SearchWidgets;
    type View = gtk::ListBox;
    type Msg = SearchEntryMsg;

    view! {
        adw::ActionRow {
            set_title: watch!(&self.value),
            set_selectable: false,
            set_activatable: true,
            connect_activated(sender) => move |_| {
                send!(sender, SearchEntryMsg::Save(Some(v.to_vec())));
            }
        }
    }

    fn pre_init() {
        let v = self
            .value
            .split('.')
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
    }

    fn position(&self, _index: &usize) {}
}

#[derive(Debug, PartialEq)]
struct SearchNameEntryOption {
    value: String,
    index: usize,
}

#[relm4::factory_prototype]
impl FactoryPrototype for SearchNameEntryOption {
    type Factory = FactoryVec<Self>;
    type Widgets = SearchNameWidgets;
    type View = gtk::ListBox;
    type Msg = SearchEntryMsg;

    view! {
        adw::ActionRow {
            set_title: watch!(&self.value),
            add_suffix = &gtk::Entry {
                set_valign: gtk::Align::Center,
                set_placeholder_text: Some("<name>"),
                set_buffer = &gtk::EntryBuffer {
                    connect_text_notify(sender) => move |x| {
                        send!(sender, SearchEntryMsg::SetName(x.text(), i));
                    }
                }
            },
            set_selectable: false,
            set_activatable: false,
        }
    }

    fn pre_init() {
        let i = self.index;
    }

    fn position(&self, _index: &usize) {}
}
