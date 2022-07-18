use adw::prelude::*;
use relm4::{*, factory::*};
use super::window::*;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct AttrPos {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub configured: bool,
    pub modified: bool,
    pub replacefor: Option<String>,
}

#[relm4::factory_prototype(pub)]
impl FactoryPrototype for AttrPos {
    type Factory = FactoryVec<Self>;
    type Widgets = AttrWidgets;
    type View = gtk::ListBox;
    type Msg = AppMsg;

    view! {
        adw::PreferencesRow {
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                append = &gtk::Label {
                    set_text: &title,
                    set_use_markup: true,
                },
                append = &gtk::Separator {
                    set_hexpand: true,
                    set_opacity: 0.0,
                },
                append = &gtk::Image {
                    set_icon_name: if self.modified { Some("system-run-symbolic") } else { Some("object-select-symbolic") },
                    set_visible: self.configured || self.modified,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn pre_init() {
        let title = if self.replacefor == Some(String::from("*")) {
            format!("[<i>{}</i>]", self.value.last().unwrap_or(&String::new()))
        } else {
            self.value.last().unwrap_or(&String::new()).to_string()
        };
    }

    fn position(&self, _index: &usize) {}
}

#[derive(Default, Debug, PartialEq)]
pub struct OptPos {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub configured: bool,
    pub modified: bool,
}

#[relm4::factory_prototype(pub)]
impl FactoryPrototype for OptPos {
    type Factory = FactoryVec<Self>;
    type Widgets = OptWidgets;
    type View = gtk::ListBox;
    type Msg = AppMsg;

    view! {
        adw::PreferencesRow {
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                append = &gtk::Label {
                    set_text: &title,
                },
                append = &gtk::Separator {
                    set_hexpand: true,
                    set_opacity: 0.0,
                },
                append = &gtk::Image {
                    set_icon_name: if self.modified { Some("system-run-symbolic") } else { Some("object-select-symbolic") },
                    set_visible: self.configured || self.modified,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn pre_init() {
        let title = self.value.last().unwrap_or(&String::new()).to_string();
    }

    fn position(&self, _index: &usize) {}
}

#[derive(Default, Debug, PartialEq)]
pub struct AttrBtn {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub opt: bool,
}

#[relm4::factory_prototype(pub)]
impl FactoryPrototype for AttrBtn {
    type Factory = FactoryVec<Self>;
    type Widgets = AttrBtnWidgets;
    type View = gtk::Box;
    type Msg = AppMsg;

    view! {
        gtk::Button {
            set_label: &title,
            connect_clicked(sender) => move |_| {
                if opt {
                    sender.send(AppMsg::OpenOption(v.to_vec(), r.to_vec())).unwrap();
                } else {
                    sender.send(AppMsg::MoveTo(v.to_vec(), r.to_vec())).unwrap();
                }
            }
        }
    }

    fn pre_init() {
        let opt = self.opt;
        let title = self.value.last().unwrap_or(&String::new()).to_string();
        let v = self.value.to_vec();
        let r = self.refvalue.to_vec();
    }

    fn position(&self, _index: &usize) {}
}
