use adw::prelude::*;
use relm4::{*, factory::*};
use super::searchpage::SearchPageMsg;

#[derive(Default, Debug, PartialEq)]
pub struct SearchOption {
    pub value: Vec<String>,
    pub configured: bool,
}

#[relm4::factory_prototype(pub)]
impl FactoryPrototype for SearchOption {
    type Factory = FactoryVecDeque<Self>;
    type Widgets = SearchWidgets;
    type View = gtk::ListBox;
    type Msg = SearchPageMsg;

    view! {
        adw::PreferencesRow {
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                append = &gtk::Label {
                    set_text: &self.value.join("."),
                },
                append = &gtk::Separator {
                    set_hexpand: true,
                    set_opacity: 0.0,
                },
                append = &gtk::Image {
                    set_icon_name: Some("object-select-symbolic"),
                    set_visible: self.configured,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn position(&self, _index: &DynamicIndex) {}
}