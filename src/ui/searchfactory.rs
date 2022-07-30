use adw::prelude::*;
use relm4::{*, factory::*};
use super::searchpage::SearchPageMsg;

#[derive(Default, Debug, PartialEq)]
pub struct SearchOption {
    pub value: Vec<String>,
    pub configured: bool,
}

#[relm4::factory(pub)]
impl FactoryComponent<gtk::ListBox, SearchPageMsg> for SearchOption {
    type Command = ();
    type CommandOutput = ();
    type InitParams = SearchOption;
    type Input = ();
    type Output = ();
    type Widgets = SearchOptionWidgets;

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
                    set_icon_name: Some("object-select-symbolic"),
                    set_visible: self.configured,
                },
            },
            set_title: &self.value.join("."),
        }
    }

    fn init_model(
        value: Self::InitParams,
        _index: &DynamicIndex,
        _input: &Sender<Self::Input>,
        _output: &Sender<Self::Output>,
    ) -> Self {
        value
    }

}