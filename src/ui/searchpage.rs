use adw::prelude::*;
use relm4::{factory::*, *};
use super::{window::*, searchfactory::SearchOption};

#[derive(Debug)]
pub enum SearchPageMsg {
    Search(String),
    OpenOption(Vec<String>),
    LoadOptions(Vec<(String, bool)>),
}

#[tracker::track]
pub struct SearchPageModel {
    pub options: Vec<(String, bool)>,
    #[tracker::no_eq]
    pub oplst: FactoryVecDeque<SearchOption>,
}

impl Model for SearchPageModel {
    type Msg = SearchPageMsg;
    type Widgets = SearchPageWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for SearchPageModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        SearchPageModel {
            options: vec![],
            oplst: FactoryVecDeque::new(),
            tracker: 0,
        }
    }

    fn update(
        &mut self,
        msg: SearchPageMsg,
        _components: &(),
        _sender: Sender<SearchPageMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        self.reset();        
        match msg {
            SearchPageMsg::Search(query) => {
                self.oplst.clear();
                let q = query.split(' ');
                let mut sortedoptions = self.options.clone();
                sortedoptions.sort();
                sortedoptions.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
                for opt in sortedoptions {
                    if q.clone().all(|part| opt.0.contains(part)) {
                        self.oplst.push_back(SearchOption {
                            value: opt.0.split('.').map(|s| s.to_string()).collect::<Vec<String>>(),
                            configured: opt.1,
                        });
                    }
                    if self.oplst.len() >= 1500 {
                        break;
                    }
                }
            }
            SearchPageMsg::OpenOption(opt) => {
                parent_sender.send(AppMsg::OpenOption(opt)).unwrap();
            }
            SearchPageMsg::LoadOptions(options) => {
                self.set_options(options);
            }
        }
    }
}

#[relm4::widget(pub)]
impl Widgets<SearchPageModel, AppModel> for SearchPageWidgets {
    view! {
        view = gtk::Stack {
            set_transition_type: gtk::StackTransitionType::Crossfade,
            add_child: options = &adw::PreferencesPage {
                set_title: "Attributes",
                add = &adw::PreferencesGroup {
                    set_title: "Options",
                    add = &gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        connect_row_activated => move |_, y| {
                            if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                let text = l.title().to_string();
                                let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                sender.send(SearchPageMsg::OpenOption(v)).unwrap();
                            }
                        },
                        factory!(model.oplst),
                    },
                }
            },
            add_child: empty = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,
                append = &adw::StatusPage {
                    set_icon_name: Some("edit-find-symbolic"),
                    set_title: "No options found!",
                    set_description: Some("Try a different search"),
                },
            }
        }
    }

    fn pre_view() {
        if model.oplst.is_empty() {
            self.view.set_visible_child(&self.empty);
        } else {
            self.view.set_visible_child(&self.options);
        }
    }
}