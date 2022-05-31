use std::collections::HashMap;
use super::about::AboutModel;
use super::preferencespage::PrefModel;
use super::preferencespage::WelcomeModel;
use super::preferencespage::WelcomeMsg;
use super::savechecking::SaveErrorModel;
use super::savechecking::SaveErrorMsg;
use super::optionpage::*;
use super::rebuild::RebuildModel;
use super::windowloading::LoadErrorModel;
use super::windowloading::WindowAsyncHandler;
use super::windowloading::WindowAsyncHandlerMsg;
use super::{
    searchpage::{SearchPageModel, SearchPageMsg},
    treefactory::*,
};
use crate::parse::config;
use crate::parse::{
    config::{opconfigured, parseconfig},
    options::*,
};
use crate::ui::about::AboutMsg;
use crate::ui::preferencespage::PrefMsg;
use crate::ui::rebuild::RebuildMsg;
use crate::ui::windowloading::LoadErrorMsg;
use adw::prelude::*;
use relm4::{
    factory::*, AppUpdate, Model, RelmApp, actions::*,
    Sender, Widgets, *,
};

#[tracker::track]
pub struct AppModel {
    pub position: Vec<String>,
    tree: AttrTree,
    #[tracker::no_eq]
    attributes: FactoryVec<AttrPos>,
    #[tracker::no_eq]
    options: FactoryVec<OptPos>,
    #[tracker::no_eq]
    posbtn: FactoryVec<AttrBtn>,
    pub conf: HashMap<String, String>,
    page: Page,
    header: HeaderBar,
    search: bool,
    busy: bool,
    pub data: HashMap<String, OptionData>,
    pub editedopts: HashMap<String, String>,
    pub configpath: String,
    pub scheme: Option<sourceview5::StyleScheme>,
    flake: Option<String>,
}

#[derive(Debug)]
pub struct LoadValues {
    pub data: HashMap<String, OptionData>,
    pub tree: AttrTree,
    pub conf: HashMap<String, String>,
    //pub attributes: Vec<AttrPos>,
    //pub options: Vec<OptPos>,
}

pub enum AppMsg {
    Welcome,
    InitialLoad(LoadValues),
    LoadError(String, String),
    TryLoad,
    Close,
    SetConfPath(String, Option<String>),
    MoveTo(Vec<String>),
    OpenOption(Vec<String>),
    ShowSearch,
    HideSearch,
    ToggleSearch,
    ShowSearchPage(String),
    HideSearchPage,
    SetBusy(bool),
    SaveError(String),
    SaveWithError,
    SaveErrorReset,
    EditOpt(String, String),
    Rebuild,
    SaveConfig,
    ResetConfig,
    ShowPrefMenu,
    SetDarkMode(bool),
}

#[derive(PartialEq, Debug)]
enum Page {
    List,
    Option,
    Loading,
}

#[derive(PartialEq, Debug)]
enum HeaderBar {
    List,
    Title,
    Search,
}

#[derive(relm4::Components)]
pub struct AppComponents {
    windowloading: RelmMsgHandler<WindowAsyncHandler, AppModel>,
    loaderror: RelmComponent<LoadErrorModel, AppModel>,
    optionpage: RelmComponent<OptPageModel, AppModel>,
    searchpage: RelmComponent<SearchPageModel, AppModel>,
    saveerror: RelmComponent<SaveErrorModel, AppModel>,
    about: RelmComponent<AboutModel, AppModel>,
    preferences: RelmComponent<PrefModel, AppModel>,
    rebuild: RelmComponent<RebuildModel, AppModel>,
    welcome: RelmComponent<WelcomeModel, AppModel>,
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: AppMsg, components: &AppComponents, sender: Sender<AppMsg>) -> bool {
        self.reset();
        match msg {
            AppMsg::Welcome => {
                send!(components.welcome.sender(), WelcomeMsg::Show);
            }
            AppMsg::InitialLoad(x) => {
                self.set_data(x.data);
                self.set_tree(x.tree);
                self.set_conf(x.conf);
                self.update_position(|x| x.clear());
                let options = self.data.keys().map(|k| {
                    let mut v = k.split('.').map(|x| x.to_string()).collect::<Vec<_>>();
                    let attr = v.pop().unwrap_or_default();
                    (k.to_string(), opconfigured(&self.conf, &v, attr))
                }).collect::<Vec<_>>();
                send!(components.searchpage.sender(), SearchPageMsg::LoadOptions(options));
                self.set_busy(false);
                send!(sender, AppMsg::MoveTo(vec![]));
            }
            AppMsg::LoadError(s, s2) => {
                self.set_busy(false);
                send!(components.loaderror.sender(), LoadErrorMsg::Show(s, s2));
            }
            AppMsg::TryLoad => {
                self.set_busy(true);
                components.windowloading.sender().blocking_send(WindowAsyncHandlerMsg::RunWindow(self.configpath.to_string())).unwrap();
            }
            AppMsg::Close => {
               relm4::gtk_application().quit();   
            }
            AppMsg::SetConfPath(s, b) => {
                self.set_busy(true);
                self.set_page(Page::Loading);
                self.set_configpath(s.clone());
                self.set_flake(b.clone());
                components.windowloading.sender().blocking_send(WindowAsyncHandlerMsg::SetConfig(s, b)).unwrap();
            }
            AppMsg::MoveTo(pos) if !self.busy => {
                match attrloc(&self.tree, pos.to_vec()) {
                    Some(x) => {
                        let mut sortedoptions = x.options.clone();
                        sortedoptions.sort();
                        self.options.clear();
                        for op in sortedoptions {
                            let mut o = pos.to_vec();
                            o.push(op.to_string());
                            self.options.push(OptPos {
                                value: o,
                                configured: opconfigured(&self.conf, &pos, op.clone()),
                                modified: opconfigured(&self.editedopts, &pos, op),
                            });
                        }
                        let mut sortedattrs = x.attributes.keys().collect::<Vec<_>>();
                        sortedattrs.sort();
                        self.attributes.clear();
                        for attr in sortedattrs {
                            let mut p = pos.to_vec();
                            p.push(attr.to_string());
                            self.attributes.push(AttrPos {
                                value: p,
                                configured: opconfigured(&self.conf, &pos, attr.to_string()),
                                modified: opconfigured(&self.editedopts, &pos, attr.to_string()),
                            });
                        }
                        if !pos.is_empty() {
                            self.posbtn.clear();
                            let mut pref = vec![];
                            for p in pos.clone() {
                                pref.push(p);
                                self.posbtn.push(AttrBtn {
                                    value: pref.to_vec(),
                                    opt: false,
                                });
                            }
                        }
                        self.set_position(pos);
                    }
                    None => {}
                }
                if self.position.is_empty() {
                    self.set_header(HeaderBar::Title);
                } else {
                    self.set_header(HeaderBar::List);
                }
                self.set_page(Page::List);
                self.update_position(|_| ());
            }
            AppMsg::OpenOption(pos) if !self.busy => {
                let d = self.data.get(&pos.join(".")).unwrap().clone();
                let conf = if let Some(x) = self.editedopts.get(&pos.join(".")) {
                    x.to_string()
                } else if let Some(n) = self.conf.get(&pos.join(".")) {
                    n.to_string()
                } else {
                    String::default()
                };
                components
                    .optionpage
                    .send(OptPageMsg::UpdateOption(d, pos.to_vec(), conf))
                    .unwrap();
                self.options.clear();
                self.attributes.clear();
                self.posbtn.clear();
                let mut pref = vec![];
                let mut p2 = pos.clone();
                p2.pop();
                for p in p2 {
                    pref.push(p);
                    self.posbtn.push(AttrBtn {
                        value: pref.to_vec(),
                        opt: false,
                    });
                }
                self.posbtn.push(AttrBtn {
                    value: pos.to_vec(),
                    opt: true,
                });
                self.set_position(pos);
                self.set_header(HeaderBar::List);
                self.set_search(false);
                self.set_page(Page::Option);
            }
            AppMsg::ShowSearch if !self.busy => self.set_header(HeaderBar::Search),
            AppMsg::HideSearch => {
                if self.position.is_empty() {
                    self.set_header(HeaderBar::Title);
                } else {
                    self.set_header(HeaderBar::List);
                }
                self.set_search(false);
            }
            AppMsg::ToggleSearch if !self.busy => {
                if self.header == HeaderBar::Search {
                    send!(sender, AppMsg::HideSearch);
                } else {
                    send!(sender, AppMsg::ShowSearch);
                }
            }
            AppMsg::ShowSearchPage(s) if !self.busy => {
                components
                    .searchpage
                    .send(SearchPageMsg::Search(s))
                    .unwrap();
                self.set_search(true)
            }
            AppMsg::HideSearchPage => self.set_search(false),
            AppMsg::SetBusy(b) => self.set_busy(b),
            AppMsg::SaveError(msg) => components.saveerror.send(SaveErrorMsg::Show(msg)).unwrap(),
            AppMsg::SaveWithError => components
                .optionpage
                .send(OptPageMsg::DoneSaving(true, "true\n".to_string()))
                .unwrap(),
            AppMsg::SaveErrorReset => components.optionpage.send(OptPageMsg::ResetConf).unwrap(),
            AppMsg::EditOpt(opt, value) => {
                self.editedopts.insert(opt, value);
            }
            AppMsg::Rebuild => {
                let conf = match config::editconfig(&self.configpath, self.editedopts.clone()) {
                    Ok(x) => x,
                    Err(e) => {
                        components.rebuild.send(RebuildMsg::FinishError(Some(format!("Error modifying configuration file.\n{}", e)))).unwrap();
                        return true;
                    }
                };
                send!(components.rebuild.sender(), RebuildMsg::Rebuild(conf, self.configpath.to_string(), self.flake.clone()));
            }
            AppMsg::ResetConfig => {
                self.update_editedopts(|x| x.clear());
                if self.page == Page::Option {
                    send!(sender, AppMsg::OpenOption(self.position.clone()));
                }
            }
            AppMsg::SaveConfig => {
                self.update_editedopts(|x| x.clear());
                let conf = match parseconfig(&self.configpath) {
                    Ok(x) => x,
                    Err(_) => {
                        send!(sender, AppMsg::LoadError(String::from("Error loading configuration file"), format!("<tt>{}</tt> may be an invalid configuration file", self.configpath)));
                        return true;
                    }
                };
                self.set_conf(conf);
            }
            AppMsg::ShowPrefMenu => {
                send!(components.preferences.sender(), PrefMsg::Show(self.configpath.to_string(), self.flake.clone()));
            }
            AppMsg::SetDarkMode(dark) => {
                let scheme = if dark {
                    "Adwaita-dark"
                } else {
                    "Adwaita"
                };
                send!(components.optionpage.sender(), OptPageMsg::SetScheme(scheme.to_string()));
                send!(components.saveerror.sender(), SaveErrorMsg::SetScheme(scheme.to_string()));
                send!(components.rebuild.sender(), RebuildMsg::SetScheme(scheme.to_string()));
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(scheme));

            }
            _ => {}
        }
        true
    }
}

#[relm4::widget(pub)]
impl Widgets<AppModel, ()> for AppWidgets {
    view! {
        main_window = adw::ApplicationWindow {
            set_default_width: 1000,
            set_default_height: 600,
            //add_css_class: "devel",
            set_sensitive: watch!(!model.busy),
            set_content: main_box = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Vertical,

                append = &adw::HeaderBar {
                    set_title_widget: headerstack = Some(&gtk::Stack) {
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        add_child: title = &gtk::Label {
                            set_label: "NixOS Configuration Editor",
                        },
                        add_child: buttons = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_halign: gtk::Align::Center,
                            add_css_class: "linked",
                            append = &gtk::Button {
                                set_icon_name: "user-home-symbolic",
                                connect_clicked(sender) => move |_| {
                                    send!(sender, AppMsg::MoveTo(vec![]))
                                },
                            },
                            factory!(model.posbtn),
                        },
                        add_child: search = &gtk::SearchEntry {
                                set_placeholder_text: Some("Search"),
                                set_halign: gtk::Align::Center,
                                set_max_width_chars: 57,
                                //set_search_delay: 500, // Change once gtk4-rs 4.8 is out
                                connect_search_changed(sender) => move |x| {
                                    if x.text().is_empty() {
                                        send!(sender, AppMsg::HideSearchPage);
                                    } else {
                                        send!(sender, AppMsg::ShowSearchPage(x.text().to_string()));
                                    }
                                },
                        },
                    },
                    pack_end: menubtn = &gtk::MenuButton {
                        set_icon_name: "view-more-symbolic",
                        set_menu_model: Some(&main_menu),
                    },
                    pack_end = &gtk::ToggleButton {
                        set_active: track!(model.changed(AppModel::position()), false),
                        set_active: track!(model.changed(AppModel::header()), model.header == HeaderBar::Search),
                        set_icon_name: "edit-find-symbolic",
                        connect_toggled(sender) => move |x| {
                            send!(sender, {
                                if x.is_active() {
                                    AppMsg::ShowSearch
                                } else {
                                    AppMsg::HideSearch
                                }
                            });
                        },
                    },
                    pack_start = &gtk::Button {
                        set_label: "Rebuild",
                        connect_clicked(sender) => move |_| {
                            send!(sender, AppMsg::Rebuild);
                        },
                    }
                },
                append: stack = &gtk::Stack {
                    set_transition_type: gtk::StackTransitionType::Crossfade,
                    add_child: loading = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_spacing: 10,
                        append = &gtk::Spinner {
                            set_spinning: true,
                            set_height_request: 80,
                        },
                        append = &gtk::Label {
                            set_label: "Loading...",
                        },
                    },
                    add_child: treeview = &adw::PreferencesPage {
                        add = &adw::PreferencesGroup {
                            set_title: "Attributes",
                            set_visible: track!(model.changed(AppModel::position()), !model.attributes.is_empty()),
                            add = &gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_selection_mode: gtk::SelectionMode::None,
                                factory!(model.attributes),
                                connect_row_activated(sender) => move |_, y| {
                                    if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                        let text = l.title().to_string();
                                        let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                        send!(sender, AppMsg::MoveTo(v))
                                    }
                                },
                            },
                        },
                        add = &adw::PreferencesGroup {
                            set_title: "Options",
                            set_visible: track!(model.changed(AppModel::position()), !model.options.is_empty()),
                            add = &gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_selection_mode: gtk::SelectionMode::None,
                                factory!(model.options),
                                connect_row_activated(sender) => move |_, y| {
                                    if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                        let text = l.title().to_string();
                                        let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                        send!(sender, AppMsg::OpenOption(v))
                                    }
                                },
                            },
                        }
                    },
                    add_titled(Some("OptPage"), "OptPage"): components.optionpage.root_widget(),
                    add_titled(Some("SearchPage"), "SearchPage"): components.searchpage.root_widget(),
                }
            },
        }
    }

    menu! {
        main_menu: {
            "Preferences" => PreferencesAction,
            "About" => AboutAction,
        }
    }

    fn pre_init() {
        components.windowloading.sender().blocking_send(WindowAsyncHandlerMsg::GetConfigPath).unwrap();
    }

    fn pre_view() {
        if !model.search {
            self.search.set_text("");
            match model.page {
                Page::List => self.stack.set_visible_child(&self.treeview),
                Page::Option => self
                    .stack
                    .set_visible_child(&self.stack.child_by_name("OptPage").unwrap()),
                Page::Loading => self.stack.set_visible_child(&self.loading),
            }
        } else {
            self.stack
                .set_visible_child(&self.stack.child_by_name("SearchPage").unwrap());
        }
        match model.header {
            HeaderBar::Title => self.headerstack.set_visible_child(&self.title),
            HeaderBar::List => self.headerstack.set_visible_child(&self.buttons),
            HeaderBar::Search => {
                self.headerstack.set_visible_child(&self.search);
                let _ = self.search.grab_focus();
            }
        }
    }

    fn post_init() {
        let app = relm4::gtk_application();
        {
            let group = RelmActionGroup::<MenuActionGroup>::new();
            let aboutsender = components.about.sender();
            let sender = sender.clone();
            let prefaction: RelmAction<PreferencesAction> = RelmAction::new_stateless(move |_| {
                send!(sender, AppMsg::ShowPrefMenu);
            });        
            let aboutaction: RelmAction<AboutAction> = RelmAction::new_stateless(move |_| {
                send!(aboutsender, AboutMsg::Show);
            });
            group.add_action(prefaction);
            group.add_action(aboutaction);
            let actions = group.into_action_group();
            main_window.insert_action_group("menu", Some(&actions));
        }
        {
            let sender = sender.clone();
            app.set_accelerators_for_action::<SearchAction>(&["<Control>f"]);
            let group = RelmActionGroup::<WindowActionGroup>::new();
            let searchaction: RelmAction<SearchAction> = RelmAction::new_stateless(move |_| {
                send!(sender, AppMsg::ToggleSearch);
            });
            group.add_action(searchaction);
            let actions = group.into_action_group();
            main_window.insert_action_group("window", Some(&actions));
        }
        {
            let sender = sender.clone();
            adw::StyleManager::default().connect_dark_notify(move |x| {
                send!(sender, AppMsg::SetDarkMode(x.is_dark()))
            });
        }
        send!(sender, AppMsg::SetDarkMode(adw::StyleManager::default().is_dark()));
    }
}

relm4::new_action_group!(MenuActionGroup, "menu");
relm4::new_stateless_action!(PreferencesAction, MenuActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, MenuActionGroup, "about");

relm4::new_action_group!(WindowActionGroup, "window");
relm4::new_stateless_action!(SearchAction, WindowActionGroup, "search");

pub fn run() {
    let model = AppModel {
        position: vec![],
        tree: AttrTree::default(),
        attributes: FactoryVec::new(),
        options: FactoryVec::new(),
        posbtn: FactoryVec::new(),
        conf: HashMap::new(),
        page: Page::Loading,
        search: false,
        busy: true,
        header: HeaderBar::Title,
        data: HashMap::new(),
        editedopts: HashMap::new(),
        configpath: String::from("/etc/nixos/configuration.nix"),
        flake: None,
        scheme: None,
        tracker: 0,
    };
    let app = RelmApp::new(model);
    app.run();
}
