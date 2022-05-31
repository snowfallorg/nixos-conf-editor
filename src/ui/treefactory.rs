use adw::prelude::*; //AdwApplicationWindowExt;
use relm4::{*, factory::*}; //{adw, gtk, send, AppUpdate, Model, RelmApp, Sender, WidgetPlus, Widgets};
use super::window::*;

#[derive(Default, Debug, PartialEq)]
pub struct AttrPos {
    pub value: Vec<String>,
    pub configured: bool,
    pub modified: bool,
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
pub struct OptPos {
    pub value: Vec<String>,
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
                    sender.send(AppMsg::OpenOption(v.to_vec())).unwrap();
                } else {
                    sender.send(AppMsg::MoveTo(v.to_vec())).unwrap();
                }
                
            }
        }
    }

    fn pre_init() {
        let opt = self.opt;
        let title = self.value.last().unwrap_or(&String::new()).to_string();
        let v = self.value.to_vec();
    }

    fn position(&self, _index: &usize) {}
}
