use super::window::*;
use adw::prelude::*;
use relm4::{factory::*, *};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct AttrPos {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub configured: bool,
    pub modified: bool,
    pub replacefor: Option<String>,
}

#[relm4::factory(pub)]
impl FactoryComponent<gtk::ListBox, AppMsg> for AttrPos {
    type Command = ();
    type CommandOutput = ();
    type InitParams = AttrPos;
    type Input = AppMsg;
    type Output = AppMsg;
    type Widgets = AttrWidgets;

    view! {
        adw::PreferencesRow {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                gtk::Label {
                    set_text: &{
                        if self.replacefor == Some(String::from("*")) {
                                    format!("[<i>{}</i>]", self.value.last().unwrap_or(&String::new()))
                                } else {
                                    self.value.last().unwrap_or(&String::new()).to_string()
                                }
                    },
                    set_use_markup: true,
                },
                gtk::Separator {
                    set_hexpand: true,
                    set_opacity: 0.0,
                },
                gtk::Image {
                    set_icon_name: if self.modified { Some("system-run-symbolic") } else { Some("object-select-symbolic") },
                    set_visible: self.configured || self.modified,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn init_model(
        parent: Self::InitParams,
        _index: &DynamicIndex,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        Self {
            value: parent.value,
            refvalue: parent.refvalue,
            configured: parent.configured,
            modified: parent.modified,
            replacefor: parent.replacefor,
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct OptPos {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub configured: bool,
    pub modified: bool,
}

#[relm4::factory(pub)]
impl FactoryComponent<gtk::ListBox, AppMsg> for OptPos {
    type Command = ();
    type CommandOutput = ();
    type InitParams = OptPos;
    type Input = AppMsg;
    type Output = AppMsg;
    type Widgets = OptWidgets;

    view! {
        adw::PreferencesRow {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                gtk::Label {
                    set_text: &{
                        self.value.last().unwrap_or(&String::new()).to_string()
                    },
                },
                gtk::Separator {
                    set_hexpand: true,
                    set_opacity: 0.0,
                },
                gtk::Image {
                    set_icon_name: if self.modified { Some("system-run-symbolic") } else { Some("object-select-symbolic") },
                    set_visible: self.configured || self.modified,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn init_model(
        parent: Self::InitParams,
        _index: &DynamicIndex,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        Self {
            value: parent.value,
            refvalue: parent.refvalue,
            configured: parent.configured,
            modified: parent.modified,
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct AttrBtn {
    pub value: Vec<String>,
    pub refvalue: Vec<String>,
    pub opt: bool,
}

#[derive(Debug)]
pub enum AttrBtnMsg {
    OpenOption(Vec<String>, Vec<String>),
    MoveTo(Vec<String>, Vec<String>),
}

#[relm4::factory(pub)]
impl FactoryComponent<gtk::Box, AppMsg> for AttrBtn {
    type Command = ();
    type CommandOutput = ();
    type InitParams = AttrBtn;
    type Input = ();
    type Output = AttrBtnMsg;
    type Widgets = AttrBtnWidgets;

    view! {
        #[name(button)]
        gtk::Button {
            set_label: self.value.last().unwrap_or(&String::new()),
            connect_clicked[output, value = self.value.clone(), refvalue = self.refvalue.clone(), opt = self.opt] => move |_| {
                if opt {
                    output.send(AttrBtnMsg::OpenOption(value.to_vec(), refvalue.to_vec()));
                } else {
                    output.send(AttrBtnMsg::MoveTo(value.to_vec(), refvalue.to_vec()));
                }
            }
        }
    }

    fn init_model(
        parent: Self::InitParams,
        _index: &DynamicIndex,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        Self {
            value: parent.value,
            refvalue: parent.refvalue,
            opt: parent.opt,
        }
    }

    fn output_to_parent_msg(output: Self::Output) -> Option<AppMsg> {
        Some(match output {
            AttrBtnMsg::OpenOption(v, r) => AppMsg::OpenOption(v, r),
            AttrBtnMsg::MoveTo(v, r) => AppMsg::MoveTo(v, r),
        })
    }
}
