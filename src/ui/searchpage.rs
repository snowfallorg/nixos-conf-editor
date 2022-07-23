use std::cmp::Ordering;

use adw::prelude::*;
use relm4::{factory::*, *};
use super::{window::*, searchfactory::SearchOption};

#[derive(Debug)]
pub enum SearchPageMsg {
    Search(String),
    OpenOption(Vec<String>, Option<Vec<String>>),
    LoadOptions(Vec<(String, bool, String)>),
}

pub struct SearchPageModel {
    pub options: Vec<(String, bool, String)>,
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
        }
    }

    fn update(
        &mut self,
        msg: SearchPageMsg,
        _components: &(),
        _sender: Sender<SearchPageMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            SearchPageMsg::Search(query) => {
                self.oplst.clear();
                let q = query.split_whitespace();
                let mut sortedoptions = self.options.clone();
                if q.clone().any(|x| x.len() > 2) {
                    sortedoptions = sortedoptions.iter().filter(|x| {
                        for part in q.clone() {
                            if x.0.to_lowercase().contains(&part.to_lowercase()) {
                                return true;
                            }
                        }
                        false
                    }).map(|x| x.to_owned()).collect::<Vec<_>>();
                    sortedoptions.sort_by(|a, b| {
                        let mut acount = 0;
                        let mut bcount = 0;
                        for part in q.clone() {
                            acount += a.0.to_lowercase().matches(&part.to_lowercase()).count();
                            bcount += b.0.to_lowercase().matches(&part.to_lowercase()).count();
                        }
                        match acount.cmp(&bcount) {
                            Ordering::Less => Ordering::Less,
                            Ordering::Greater => Ordering::Greater,
                            Ordering::Equal => a.0.len().cmp(&b.0.len()),
                        }
                    });
                } else {
                    sortedoptions.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
                }
                for opt in sortedoptions {
                    if q.clone().all(|part| opt.0.to_lowercase().contains(&part.to_lowercase()) || if q.clone().any(|x| x.len() > 2) { opt.2.to_lowercase().contains(&part.to_lowercase()) } else { false }) {
                        self.oplst.push_back(SearchOption {
                            value: opt.0.split('.').map(|s| s.to_string()).collect::<Vec<String>>(),
                            configured: opt.1,
                        });
                    }
                    if self.oplst.len() >= 1000 {
                        break;
                    }
                }
            }
            SearchPageMsg::OpenOption(opt, refpos) => {
                if opt.contains(&String::from("*")) || opt.contains(&String::from("<name>")) {
                    send!(parent_sender, AppMsg::ShowSearchPageEntry(opt));
                } else {
                    parent_sender.send(AppMsg::OpenOption(opt.clone(), if let Some(x) = refpos { x } else { opt })).unwrap();
                }
            }
            SearchPageMsg::LoadOptions(options) => {
                self.options = options;
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
                                sender.send(SearchPageMsg::OpenOption(v, None)).unwrap();
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
