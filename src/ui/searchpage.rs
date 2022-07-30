use std::cmp::Ordering;

use super::{searchfactory::SearchOption, window::*};
use adw::prelude::*;
use relm4::{factory::*, *};

#[derive(Debug)]
pub enum SearchPageMsg {
    Search(String),
    OpenOption(Vec<String>, Option<Vec<String>>),
    LoadOptions(Vec<(String, bool, String)>),
}

pub struct SearchPageModel {
    pub options: Vec<(String, bool, String)>,
    pub oplst: FactoryVecDeque<gtk::ListBox, SearchOption, SearchPageMsg>,
}

#[relm4::component(pub)]
impl SimpleComponent for SearchPageModel {
    type InitParams = ();
    type Input = SearchPageMsg;
    type Output = AppMsg;
    type Widgets = SearchPageWidgets;

    view! {
        view = gtk::Stack {
            set_transition_type: gtk::StackTransitionType::Crossfade,
            #[name(options)]
            adw::PreferencesPage {
                set_title: "Attributes",
                add = &adw::PreferencesGroup {
                    set_title: "Options",
                    #[local_ref]
                    add = oplstbox -> gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        connect_row_activated[sender] => move |_, y| {
                            if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                let text = l.title().to_string();
                                let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                sender.input(SearchPageMsg::OpenOption(v, None));
                            }
                        },
                    },
                }
            },
            #[name(empty)]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,
                adw::StatusPage {
                    set_icon_name: Some("edit-find-symbolic"),
                    set_title: "No options found!",
                    set_description: Some("Try a different search"),
                },
            }
        }
    }

    fn pre_view() {
        if model.oplst.is_empty() {
            view.set_visible_child(empty);
        } else {
            view.set_visible_child(options);
        }
    }

    fn init(
        _value: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SearchPageModel {
            options: vec![],
            oplst: FactoryVecDeque::new(gtk::ListBox::new(), &sender.input),
        };
        let oplstbox = model.oplst.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: &ComponentSender<Self>) {
        let mut oplst_guard = self.oplst.guard();
        match msg {
            SearchPageMsg::Search(query) => {
                oplst_guard.clear();
                let q = query.split_whitespace();
                let mut sortedoptions = self.options.clone();
                if q.clone().any(|x| x.len() > 2) {
                    sortedoptions = sortedoptions
                        .iter()
                        .filter(|x| {
                            for part in q.clone() {
                                if x.0.to_lowercase().contains(&part.to_lowercase()) {
                                    return true;
                                }
                            }
                            false
                        })
                        .map(|x| x.to_owned())
                        .collect::<Vec<_>>();
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
                    if q.clone().all(|part| {
                        opt.0.to_lowercase().contains(&part.to_lowercase())
                            || if q.clone().any(|x| x.len() > 2) {
                                opt.2.to_lowercase().contains(&part.to_lowercase())
                            } else {
                                false
                            }
                    }) {
                        oplst_guard.push_back(SearchOption {
                            value: opt
                                .0
                                .split('.')
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>(),
                            configured: opt.1,
                        });
                    }
                    if oplst_guard.len() >= 1000 {
                        break;
                    }
                }
            }
            SearchPageMsg::OpenOption(opt, refpos) => {
                if opt.contains(&String::from("*")) || opt.contains(&String::from("<name>")) {
                    sender.output(AppMsg::ShowSearchPageEntry(opt));
                } else {
                    sender.output(AppMsg::OpenOption(
                        opt.clone(),
                        if let Some(x) = refpos { x } else { opt },
                    ));
                }
            }
            SearchPageMsg::LoadOptions(options) => {
                self.options = options;
            }
        }
    }
}
