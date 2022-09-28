use adw::prelude::*;
use relm4::{*, factory::*};
use super::searchpage::SearchPageMsg;

#[derive(Default, Debug, PartialEq)]
pub struct SearchOption {
    pub value: Vec<String>,
    pub configured: bool,
    pub modified: bool,
}

#[relm4::factory(pub)]
impl FactoryComponent for SearchOption {
    type Init = SearchOption;
    type Input = ();
    type Output = ();
    type Widgets = SearchOptionWidgets;
    type ParentWidget = gtk::ListBox;
    type ParentMsg = SearchPageMsg;
    type CommandOutput = ();

    view! {
        adw::PreferencesRow {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_margin_all: 15,
                gtk::Label {
                    set_text: &self.value.join("."),
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
        value: Self::Init,
        _index: &DynamicIndex,
        _sender: FactoryComponentSender<Self>,
    ) -> Self {
        value
    }

}