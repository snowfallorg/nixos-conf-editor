use super::about::AboutPageModel;
use super::nameentry::NameEntryModel;
use super::optionpage::*;
use super::preferencespage::PreferencesPageModel;
use super::rebuild::RebuildModel;
use super::savechecking::SaveErrorModel;
use super::savechecking::SaveErrorMsg;
use super::searchentry::SearchEntryModel;
use super::welcome::WelcomeModel;
use super::welcome::WelcomeMsg;
use super::windowloading::LoadErrorModel;
use super::windowloading::WindowAsyncHandler;
use super::windowloading::WindowAsyncHandlerMsg;
use super::{
    searchpage::{SearchPageModel, SearchPageMsg},
    treefactory::*,
};
use crate::parse::config;
use crate::parse::config::getarrvals;
use crate::parse::config::getconfvals;
use crate::parse::config::opconfigured2;
use crate::parse::config::readval;
use crate::parse::preferences::getconfig;
use crate::parse::{
    config::{opconfigured, parseconfig},
    options::*,
};
use crate::ui::nameentry::NameEntryMsg;
use crate::ui::preferencespage::PreferencesPageMsg;
use crate::ui::quitdialog::{QuitCheckModel, QuitCheckMsg};
use crate::ui::rebuild::RebuildMsg;
use crate::ui::searchentry::SearchEntryMsg;
use crate::ui::windowloading::LoadErrorMsg;
use adw::prelude::*;
use log::*;
use nix_data::config::configfile::NixDataConfig;
use relm4::gtk::glib::object::Cast;
use relm4::{actions::*, factory::*, *};
use std::collections::HashMap;
use std::convert::identity;

#[tracker::track]
pub struct AppModel {
    mainwindow: adw::ApplicationWindow,
    pub position: Vec<String>,
    pub refposition: Vec<String>,
    tree: AttrTree,
    #[tracker::no_eq]
    attributes: FactoryVecDeque<AttrPos>,
    #[tracker::no_eq]
    options: FactoryVecDeque<OptPos>,
    #[tracker::no_eq]
    posbtn: FactoryVecDeque<AttrBtn>,
    pub conf: HashMap<String, String>,
    page: Page,
    header: HeaderBar,
    search: bool,
    busy: bool,
    pub data: HashMap<String, OptionData>,
    pub editedopts: HashMap<String, String>,
    nameattrs: HashMap<String, Vec<String>>,
    starattrs: HashMap<String, usize>,
    pub scheme: Option<sourceview5::StyleScheme>,
    fieldreplace: HashMap<usize, String>,
    nameorstar: AddAttrOptions,
    config: NixDataConfig,
    modifiedonly: bool,

    // Components
    #[tracker::no_eq]
    preferencespage: Controller<PreferencesPageModel>,
    #[tracker::no_eq]
    windowloading: WorkerController<WindowAsyncHandler>,
    #[tracker::no_eq]
    loaderror: Controller<LoadErrorModel>,
    #[tracker::no_eq]
    optionpage: Controller<OptPageModel>,
    #[tracker::no_eq]
    searchpage: Controller<SearchPageModel>,
    #[tracker::no_eq]
    saveerror: Controller<SaveErrorModel>,
    #[tracker::no_eq]
    rebuild: Controller<RebuildModel>,
    #[tracker::no_eq]
    welcome: Controller<WelcomeModel>,
    #[tracker::no_eq]
    nameentry: Controller<NameEntryModel>,
    #[tracker::no_eq]
    searchpageentry: Controller<SearchEntryModel>,
    #[tracker::no_eq]
    quitdialog: Controller<QuitCheckModel>,
}

#[derive(Debug)]
pub struct LoadValues {
    pub data: HashMap<String, OptionData>,
    pub tree: AttrTree,
    pub conf: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
enum AddAttrOptions {
    Star,
    Name,
    None,
}

#[derive(Debug)]
pub enum AppMsg {
    Welcome,
    InitialLoad(LoadValues),
    LoadError(String, String),
    TryLoad,
    Close,
    SetConfig(NixDataConfig),
    MoveTo(Vec<String>, Vec<String>),
    MoveToSelf,
    MoveToRow(Vec<String>),
    OpenOption(Vec<String>, Vec<String>),
    OpenOptionRow(Vec<String>),
    ShowSearch,
    HideSearch,
    ToggleSearch,
    ShowSearchPage(String),
    HideSearchPage,
    ShowSearchPageEntry(Vec<String>),
    SetBusy(bool),
    SaveError(String),
    SaveWithError,
    SaveErrorReset,
    EditOpt(String, String),
    Rebuild,
    SaveConfig,
    ResetConfig,
    ShowPrefMenu,
    ShowPrefMenuErr,
    SetDarkMode(bool),
    AddAttr,
    AddNameAttr(Option<String>, String),
    AddStar(String),
    OpenSearchOption(Vec<String>, Vec<String>),
    SaveQuit,
    ShowAboutPage,
    SetModifiedOnly(bool),
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

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    view! {
        main_window = adw::ApplicationWindow {
            set_default_width: 1000,
            set_default_height: 650,
            #[watch]
            set_sensitive: !model.busy,
            connect_close_request[sender] => move |_| {
                debug!("Caught close request");
                sender.input(AppMsg::Close);
                gtk::Inhibit(true)
            },
            #[wrap(Some)]
            set_content: main_box = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget: headerstack = &gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        #[name(title)]
                        gtk::Label {
                            set_label: "Configuration Editor",
                        },

                        #[name(buttons)]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_halign: gtk::Align::Center,
                            add_css_class: "linked",

                            #[local_ref]
                            buttonsbox -> gtk::Box {
                                #[local]
                                prepend = &homebtn -> gtk::Button {
                                    set_icon_name: "user-home-symbolic",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AppMsg::MoveTo(vec![], vec![]))
                                    }
                                },
                                set_orientation: gtk::Orientation::Horizontal,
                                add_css_class: "linked",
                            }
                        },

                        #[name(search)]
                        gtk::SearchEntry {
                                add_css_class: "inline",
                                set_placeholder_text: Some("Search"),
                                set_halign: gtk::Align::Center,
                                set_max_width_chars: 57,
                                //set_search_delay: 500, // Change once gtk4-rs 4.8 is out
                                connect_search_changed[sender] => move |x| {
                                    if x.text().is_empty() {
                                        sender.input(AppMsg::HideSearchPage);
                                    } else {
                                        sender.input(AppMsg::ShowSearchPage(x.text().to_string()));
                                    }
                                },
                        },
                    },
                    pack_end: menubtn = &gtk::MenuButton {
                        set_icon_name: "open-menu-symbolic",
                        set_menu_model: Some(&main_menu),
                    },
                    pack_end = &gtk::ToggleButton {
                        #[track(model.changed(AppModel::position()))]
                        set_active: false,
                        #[track(model.changed(AppModel::header()))]
                        set_active: model.header == HeaderBar::Search,
                        set_icon_name: "edit-find-symbolic",
                        connect_toggled[sender] => move |x| {
                            sender.input({
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
                        connect_clicked[sender] => move |_| {
                            sender.input(AppMsg::Rebuild);
                        },
                    }
                },
                #[name(stack)]
                gtk::Stack {
                    #[name(loading)]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_spacing: 10,
                        gtk::Spinner {
                            #[watch]
                            set_spinning: true,
                            set_height_request: 80,
                        },
                        gtk::Label {
                            set_label: "Loading...",
                        },
                    },
                    #[name(treeview)]
                    adw::PreferencesPage {
                        add: attrgroup = &adw::PreferencesGroup {
                            set_title: "Attributes",
                            #[track(model.changed(AppModel::position()))]
                            set_visible: !model.attributes.is_empty() || model.nameorstar != AddAttrOptions::None,
                            #[local_ref]
                            add = attrlistbox -> gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_selection_mode: gtk::SelectionMode::None,
                                append: addrow = &adw::PreferencesRow { // Change to suffix once libadwaita-rs 0.2 is out
                                    #[track(model.changed(AppModel::nameorstar()))]
                                    set_visible: model.nameorstar != AddAttrOptions::None,
                                    set_title: "<ADD>",
                                    #[wrap(Some)]
                                    set_child = &gtk::Box {
                                        set_margin_all: 15,
                                        gtk::Image {
                                            set_halign: gtk::Align::Center,
                                            set_hexpand: true,
                                            set_icon_name: Some("list-add-symbolic"),
                                            add_css_class: "accent",
                                        }
                                    }
                                },
                                connect_row_activated[sender] => move |_, y| {
                                    if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                        if l.title() != "<ADD>" {
                                            let text = l.title().to_string();
                                            let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                            sender.input(AppMsg::MoveToRow(v));
                                        } else {
                                            sender.input(AppMsg::AddAttr);
                                        }
                                    }
                                },
                            },
                        },
                        add = &adw::PreferencesGroup {
                            set_title: "Options",
                            #[track(model.changed(AppModel::position()))]
                            set_visible: !model.options.is_empty(),
                            #[local_ref]
                            add = optlistbox -> gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_selection_mode: gtk::SelectionMode::None,
                                connect_row_activated[sender] => move |_, y| {
                                     if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                        let text = l.title().to_string();
                                        let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                        sender.input(AppMsg::OpenOptionRow(v))
                                     }
                                },
                            },
                        }
                    },
                    #[name(optpage)]
                    gtk::Box {
                        append: model.optionpage.widget()
                    },
                    add_titled: (model.searchpage.widget(), Some("SearchPage"), "SearchPage")
                }
            },
        }
    }

    menu! {
        main_menu: {
            "Modified only" => ModifiedAction(1_u8),
            "Preferences" => PreferencesAction,
            "About" => AboutAction,
        }
    }

    fn pre_view() {
        buttonsbox.remove(homebtn);
        buttonsbox.prepend(homebtn);
        if !model.search {
            search.set_text("");
            match model.page {
                Page::List => stack.set_visible_child(treeview),
                Page::Option => stack.set_visible_child(optpage), //stack.set_visible_child(&stack.child_by_name("OptPage").unwrap()),
                Page::Loading => stack.set_visible_child(loading),
            }
        } else {
            stack.set_visible_child(&stack.child_by_name("SearchPage").unwrap());
        }
        match model.header {
            HeaderBar::Title => headerstack.set_visible_child(title),
            HeaderBar::List => headerstack.set_visible_child(buttons),
            HeaderBar::Search => {
                headerstack.set_visible_child(search);
                let _ = search.grab_focus();
            }
        }
    }

    fn init(
        (): Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = getconfig();
        warn!("GOT CONFIG: {:?}", config);

        let preferencespage = PreferencesPageModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let windowloading = WindowAsyncHandler::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);
        let loaderror = LoadErrorModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let optionpage = OptPageModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let searchpage = SearchPageModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let saveerror = SaveErrorModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let rebuild = RebuildModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let welcome = WelcomeModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let nameentry = NameEntryModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let searchpageentry = SearchEntryModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        let quitdialog = QuitCheckModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);

        windowloading.emit(WindowAsyncHandlerMsg::GetConfigPath(config.clone()));

        let model = AppModel {
            mainwindow: root.clone(),
            position: vec![],
            refposition: vec![],
            tree: AttrTree::default(),
            attributes: FactoryVecDeque::new(gtk::ListBox::new(), sender.input_sender()),
            options: FactoryVecDeque::new(gtk::ListBox::new(), sender.input_sender()),
            posbtn: FactoryVecDeque::new(
                gtk::Box::new(gtk::Orientation::Horizontal, 0),
                sender.input_sender(),
            ),
            conf: HashMap::new(),
            page: Page::Loading,
            search: false,
            busy: true,
            header: HeaderBar::Title,
            data: HashMap::new(),
            editedopts: HashMap::new(),
            nameattrs: HashMap::new(),
            starattrs: HashMap::new(),
            config: if let Some(cfg) = config {
                cfg
            } else {
                NixDataConfig {
                    systemconfig: Some(String::from("/etc/nixos/configuration.nix")),
                    flake: None,
                    flakearg: None,
                    generations: None,
                }
            },
            modifiedonly: false,
            scheme: None,
            fieldreplace: HashMap::new(),
            nameorstar: AddAttrOptions::None,
            preferencespage,
            windowloading,
            loaderror,
            optionpage,
            searchpage,
            saveerror,
            rebuild,
            welcome,
            nameentry,
            searchpageentry,
            quitdialog,
            tracker: 0,
        };
        let attrlistbox = model.attributes.widget();
        let optlistbox = model.options.widget();
        let buttonsbox = model.posbtn.widget();
        let homebtn = gtk::Button::new();
        let widgets = view_output!();

        {
            let group = RelmActionGroup::<MenuActionGroup>::new();
            let prefsender = sender.clone();
            let prefaction: RelmAction<PreferencesAction> = RelmAction::new_stateless(move |_| {
                prefsender.input(AppMsg::ShowPrefMenu);
            });

            let aboutsender = sender.clone();
            let aboutaction: RelmAction<AboutAction> = RelmAction::new_stateless(move |_| {
                aboutsender.input(AppMsg::ShowAboutPage);
            });

            let modifiedsender = sender.clone();
            let modifiedaction: RelmAction<ModifiedAction> =
                RelmAction::new_stateful_with_target_value(&0, move |_, state, _value| {
                    *state ^= 1;
                    let modified = *state != 0;
                    modifiedsender.input(AppMsg::SetModifiedOnly(modified));
                });
            group.add_action(&prefaction);
            group.add_action(&aboutaction);
            group.add_action(&modifiedaction);
            let actions = group.into_action_group();
            widgets
                .main_window
                .insert_action_group("menu", Some(&actions));
        }
        {
            let sender = sender.clone();
            let group = RelmActionGroup::<WindowActionGroup>::new();
            let searchaction: RelmAction<SearchAction> = RelmAction::new_stateless(move |_| {
                sender.input(AppMsg::ToggleSearch);
            });
            group.add_action(&searchaction);
            let actions = group.into_action_group();
            widgets
                .main_window
                .insert_action_group("window", Some(&actions));
        }
        {
            let sender = sender.clone();
            adw::StyleManager::default()
                .connect_dark_notify(move |x| sender.input(AppMsg::SetDarkMode(x.is_dark())));
        }
        sender.input(AppMsg::SetDarkMode(adw::StyleManager::default().is_dark()));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            AppMsg::Welcome => {
                info!("Received AppMsg::Welcome");
                self.welcome.emit(WelcomeMsg::Show);
            }
            AppMsg::InitialLoad(x) => {
                info!("Received AppMsg::InitialLoad");
                self.set_data(x.data);
                self.set_tree(x.tree);
                self.set_conf(x.conf);
                // trace!("CONF:\n{:#?}", self.conf);
                self.update_position(|x| x.clear());
                let options = self
                    .data
                    .iter()
                    .map(|(k, data)| {
                        let mut v = k.split('.').map(|x| x.to_string()).collect::<Vec<_>>();
                        let attr = v.pop().unwrap_or_default();
                        (
                            k.to_string(),
                            opconfigured(&self.conf, &v, attr),
                            data.description
                                .as_string()
                                .map(|x| x.to_string())
                                .unwrap_or_default(),
                        )
                    })
                    .collect::<Vec<_>>();
                self.searchpage.emit(SearchPageMsg::LoadOptions(options));
                self.set_busy(false);
                sender.input(AppMsg::MoveTo(vec![], vec![]));
            }
            AppMsg::LoadError(s, s2) => {
                info!("Received AppMsg::LoadError");
                self.set_busy(false);
                self.loaderror.emit(LoadErrorMsg::Show(s, s2));
            }
            AppMsg::TryLoad => {
                info!("Received AppMsg::TryLoad");
                self.set_busy(true);
                self.windowloading.emit(WindowAsyncHandlerMsg::RunWindow(
                    self.config.systemconfig.as_ref().unwrap().to_string(),
                ));
            }
            AppMsg::Close => {
                info!("Received AppMsg::Close");
                if self.editedopts.is_empty() {
                    relm4::main_application().quit();
                } else {
                    self.quitdialog.emit(QuitCheckMsg::Show);
                }
            }
            AppMsg::SetConfig(cfg) => {
                info!("Received AppMsg::SetConfPath");
                self.set_busy(true);
                self.set_page(Page::Loading);
                self.set_config(cfg.clone());
                // self.set_configpath(s.clone());
                // self.set_flake(b.clone());
                self.windowloading
                    .emit(WindowAsyncHandlerMsg::SetConfig(cfg));
            }
            AppMsg::MoveToSelf => {
                info!("Received AppMsg::MoveToSelf");
                sender.input(AppMsg::MoveTo(
                    self.position.clone(),
                    self.refposition.clone(),
                ));
            }
            AppMsg::MoveToRow(pos) => {
                info!("Received AppMsg::MoveToRow");

                let attributes_guard = self.attributes.guard();
                let mut attrvec = vec![];
                for i in 0..attributes_guard.len() {
                    attrvec.push(attributes_guard.get(i).unwrap());
                }

                match attrvec.iter().find(|x| x.value == pos) {
                    Some(x) => {
                        debug!("FOUND ATTR: {:?}", x);
                        sender.input(AppMsg::MoveTo(x.value.to_vec(), x.refvalue.to_vec()));
                    }
                    None => {
                        error!("Received AppMsg::MoveToRow, but no attribute found");
                    }
                }
            }
            AppMsg::MoveTo(pos, newref) if !self.busy => {
                info!("Received AppMsg::MoveTo");
                debug!("Moving to {:?}", pos);
                let mut p = pos.clone();
                let mut attributes_guard = self.attributes.guard();
                let mut options_guard = self.options.guard();
                let mut posbtn_guard = self.posbtn.guard();
                let mut attrvec = vec![];
                for i in 0..attributes_guard.len() {
                    attrvec.push(attributes_guard.get(i).unwrap().clone());
                }
                if let Some(x) = attrvec.iter().find(|x| x.value.eq(&pos)) {
                    if let Some(y) = &x.replacefor {
                        p.pop();
                        p.push(String::from(y));
                    }
                }

                debug!("NEW REFPOSITON: {:?}", newref);

                if let Some(x) = attrloc(&self.tree, newref.to_vec()) {
                    let mut sortedoptions = x.options.clone();
                    sortedoptions.sort();
                    options_guard.clear();
                    for op in sortedoptions {
                        let configured = if pos.eq(&newref) {
                            opconfigured(&self.conf, &pos, op.clone())
                        } else {
                            opconfigured2(
                                self.config.systemconfig.as_ref().unwrap(),
                                &pos,
                                &newref,
                                op.clone(),
                            )
                        };
                        let modified = opconfigured(&self.editedopts, &pos, op.clone());
                        if self.modifiedonly && !(configured || modified) {
                            continue;
                        }
                        let mut o = pos.to_vec();
                        let mut r = newref.to_vec();
                        o.push(op.to_string());
                        r.push(op.to_string());
                        options_guard.push_back(OptPos {
                            value: o,
                            refvalue: r,
                            configured,
                            modified,
                        });
                    }
                    attributes_guard.clear();
                    let mut attributes = Vec::new();
                    let mut hasnameorstar = AddAttrOptions::None;
                    debug!("ATTRS {:?}", x.attributes.keys());
                    for attr in x.attributes.keys().collect::<Vec<_>>() {
                        if attr == "<name>" {
                            debug!("FOUND <name> ATTR");
                            hasnameorstar = AddAttrOptions::Name;
                            let v = getconfvals(&self.conf, &pos);
                            for x in v {
                                let mut p = pos.clone();
                                let mut r = newref.clone();
                                p.push(x.clone());
                                r.push(String::from("<name>"));
                                attributes.push(AttrPos {
                                    value: p,
                                    refvalue: r,
                                    configured: true,
                                    modified: opconfigured(&self.editedopts, &pos, x),
                                    replacefor: Some(String::from("<name>")),
                                })
                            }
                            let addedvals = self.nameattrs.get(&pos.join("."));
                            if let Some(x) = addedvals {
                                for a in x {
                                    let mut p = pos.clone();
                                    let mut r = newref.clone();
                                    p.push(a.clone());
                                    r.push(String::from("<name>"));
                                    attributes.push(AttrPos {
                                        value: p,
                                        refvalue: r,
                                        configured: false,
                                        modified: opconfigured(
                                            &self.editedopts,
                                            &pos,
                                            a.to_string(),
                                        ),
                                        replacefor: Some(String::from("<name>")),
                                    })
                                }
                            }
                        } else if attr == "*" {
                            debug!("FOUND * ATTR");
                            hasnameorstar = AddAttrOptions::Star;
                            let v = getarrvals(self.config.systemconfig.as_ref().unwrap(), &pos);
                            debug!("V: {:?}", v);
                            for i in 0..v.len() {
                                let mut p = pos.clone();
                                let mut r = newref.clone();
                                p.push(i.to_string());
                                r.push(String::from("*"));
                                attributes.push(AttrPos {
                                    value: p,
                                    refvalue: r,
                                    configured: true,
                                    modified: opconfigured(&self.editedopts, &pos, i.to_string()),
                                    replacefor: Some(String::from("*")),
                                })
                            }
                            let s = self.starattrs.get(&pos.join(".")).unwrap_or(&0);
                            for i in v.len()..s + v.len() {
                                let mut p = pos.clone();
                                let mut r = newref.clone();
                                p.push(i.to_string());
                                r.push(String::from("*"));
                                attributes.push(AttrPos {
                                    value: p,
                                    refvalue: r,
                                    configured: false,
                                    modified: opconfigured(&self.editedopts, &pos, i.to_string()),
                                    replacefor: Some(String::from("*")),
                                })
                            }
                        } else {
                            let configured = if pos.eq(&newref) {
                                opconfigured(&self.conf, &pos, attr.to_string())
                            } else {
                                opconfigured2(
                                    self.config.systemconfig.as_ref().unwrap(),
                                    &pos,
                                    &newref,
                                    attr.to_string(),
                                )
                            };
                            let modified =
                                opconfigured(&self.editedopts, &newref, attr.to_string());
                            if self.modifiedonly && !(configured || modified) {
                                continue;
                            }
                            let mut p = pos.to_vec();
                            let mut r = newref.to_vec();
                            p.push(attr.to_string());
                            r.push(attr.to_string());
                            attributes.push(AttrPos {
                                value: p,
                                refvalue: r,
                                configured,
                                modified,
                                replacefor: None,
                            });
                        }
                    }
                    if !pos.is_empty() {
                        posbtn_guard.clear();
                        let mut pref = vec![];
                        let mut rref = vec![];
                        for i in 0..pos.len() {
                            pref.push(pos[i].clone());
                            rref.push(newref[i].clone());
                            posbtn_guard.push_back(AttrBtn {
                                value: pref.to_vec(),
                                refvalue: rref.to_vec(),
                                opt: false,
                            });
                        }
                    }

                    let mut x = attributes.to_vec();
                    x.sort_by(|x, y| x.value.cmp(&y.value));
                    for attr in x {
                        attributes_guard.push_back(attr.clone());
                    }
                    debug!("Setting HNOS {:?}", hasnameorstar);
                    self.nameorstar = hasnameorstar;
                    self.position = pos;
                    self.refposition = newref;
                }
                if self.position.is_empty() {
                    self.header = HeaderBar::Title;
                } else {
                    self.header = HeaderBar::List;
                }

                attributes_guard.drop();
                options_guard.drop();
                posbtn_guard.drop();
                self.set_page(Page::List);
                self.update_position(|_| ());
                self.update_refposition(|_| ());
                self.update_nameorstar(|_| ());
                self.update_header(|_| ());
            }
            AppMsg::OpenOption(pos, newref) if !self.busy => {
                info!("Received AppMsg::OpenOption");
                trace!("NEW REFPOSITON: {:?}", newref);
                let d = match self.data.get(&newref.join(".")) {
                    Some(x) => x,
                    None => {
                        error!("No data for {:?}", newref);
                        return;
                    }
                };

                let conf = if let Some(x) = self.editedopts.get(&pos.join(".")) {
                    trace!("EDITED");
                    x.to_string()
                } else if let Some(n) = self.conf.get(&pos.join(".")) {
                    trace!("CONFIGURED");
                    n.to_string()
                } else if let Ok(v) = readval(
                    self.config.systemconfig.as_ref().unwrap(),
                    &pos.join("."),
                    &newref.join("."),
                ) {
                    trace!("READ");
                    v
                } else {
                    trace!("EMPTY");
                    String::default()
                };

                self.optionpage.emit(OptPageMsg::UpdateOption(
                    Box::new(d.clone()),
                    pos.to_vec(),
                    newref.to_vec(),
                    conf,
                    self.data
                        .keys()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>(),
                ));

                let mut attributes_guard = self.attributes.guard();
                let mut options_guard = self.options.guard();
                let mut posbtn_guard = self.posbtn.guard();

                options_guard.clear();
                attributes_guard.clear();
                posbtn_guard.clear();
                let mut pref = vec![];
                let mut rref = vec![];
                let mut p2 = pos.clone();
                let mut r2 = newref.clone();
                p2.pop();
                r2.pop();
                for i in 0..p2.len() {
                    pref.push(p2[i].clone());
                    rref.push(r2[i].clone());
                    posbtn_guard.push_back(AttrBtn {
                        value: pref.to_vec(),
                        refvalue: rref.to_vec(),
                        opt: false,
                    });
                }
                posbtn_guard.push_back(AttrBtn {
                    value: pos.to_vec(),
                    refvalue: newref.to_vec(),
                    opt: true,
                });

                attributes_guard.drop();
                options_guard.drop();
                posbtn_guard.drop();

                self.set_position(pos);
                self.set_refposition(newref);
                self.set_header(HeaderBar::List);
                self.set_page(Page::Option);
                self.set_search(false);
            }
            AppMsg::OpenOptionRow(pos) => {
                info!("Received AppMsg::OpenOptionRow");
                let options_guard = self.options.guard();
                let mut optvec = vec![];
                for i in 0..options_guard.len() {
                    optvec.push(options_guard.get(i).unwrap())
                }

                match optvec.iter().find(|x| x.value == pos) {
                    Some(x) => {
                        sender.input(AppMsg::OpenOption(x.value.to_vec(), x.refvalue.to_vec()));
                    }
                    None => {
                        error!("Received AppMsg::OpenOptionRow, but no options found");
                    }
                }
            }
            AppMsg::ShowSearch if !self.busy => {
                info!("Received AppMsg::ShowSearch");
                self.set_header(HeaderBar::Search)
            }
            AppMsg::HideSearch => {
                info!("Received AppMsg::HideSearch");
                if self.position.is_empty() {
                    self.set_header(HeaderBar::Title);
                } else {
                    self.set_header(HeaderBar::List);
                }
                self.set_search(false);
            }
            AppMsg::ToggleSearch if !self.busy => {
                info!("Received AppMsg::ToggleSearch");
                if self.header == HeaderBar::Search {
                    sender.input(AppMsg::HideSearch);
                } else {
                    sender.input(AppMsg::ShowSearch);
                }
            }
            AppMsg::ShowSearchPage(s) if !self.busy => {
                info!("Received AppMsg::ShowSearchPage");
                self.searchpage
                    .emit(SearchPageMsg::Search(s, self.editedopts.clone()));
                self.set_search(true)
            }
            AppMsg::HideSearchPage => {
                info!("Received AppMsg::HideSearchPage");
                if self.page == Page::List {
                    sender.input(AppMsg::MoveToSelf);
                }
                self.set_search(false)
            }
            AppMsg::ShowSearchPageEntry(pos) => {
                info!("Received AppMsg::ShowSearchPageEntry");
                // Input a string of the form "service.<name>.groups.*.uid" and return a vector of all possible existing options for that string.
                fn getposdata(
                    pos: &Vec<String>,
                    conf: &HashMap<String, String>,
                    nameattrs: &HashMap<String, Vec<String>>,
                    starattrs: &HashMap<String, usize>,
                    configpath: &str,
                ) -> Vec<Vec<String>> {
                    for i in 0..pos.len() {
                        if pos[i] == "<name>" {
                            let mut possiblevals = getconfvals(conf, &pos[..i]);
                            if let Some(x) = nameattrs.get(&pos[..i].join(".")) {
                                possiblevals.append(&mut x.to_vec());
                            }
                            let mut out = vec![];
                            for x in possiblevals {
                                let mut newpos = pos.clone();
                                newpos[i] = x.clone();
                                out.append(&mut getposdata(
                                    &newpos, conf, nameattrs, starattrs, configpath,
                                ));
                            }
                            return out;
                        } else if pos[i] == "*" {
                            let v = getarrvals(configpath, &pos[..i]);
                            let mut n = v.len();
                            if let Some(x) = starattrs.get(&pos[..i].join(".")) {
                                n += *x;
                            }
                            let mut out = vec![];
                            for j in 0..n {
                                let mut newpos = pos.clone();
                                newpos[i] = j.to_string();
                                out.append(&mut getposdata(
                                    &newpos, conf, nameattrs, starattrs, configpath,
                                ));
                            }
                            return out;
                        }
                    }
                    vec![pos.to_vec()]
                }

                let data = getposdata(
                    &pos,
                    &self.conf,
                    &self.nameattrs,
                    &self.starattrs,
                    self.config.systemconfig.as_ref().unwrap(),
                )
                .iter()
                .map(|x| x.join("."))
                .collect::<Vec<String>>();
                self.searchpageentry.emit(SearchEntryMsg::Show(pos, data));
            }
            AppMsg::SetBusy(b) => {
                info!("Received AppMsg::SetBusy");
                self.set_busy(b)
            }
            AppMsg::SaveError(msg) => {
                info!("Received AppMsg::SaveError");
                self.saveerror.emit(SaveErrorMsg::Show(msg))
            }
            AppMsg::SaveWithError => {
                info!("Received AppMsg::SaveWithError");
                self.optionpage
                    .emit(OptPageMsg::DoneSaving(true, "true\n".to_string()))
            }
            AppMsg::SaveErrorReset => {
                info!("Received AppMsg::SaveErrorReset");
                self.optionpage.emit(OptPageMsg::ResetConf)
            }
            AppMsg::EditOpt(opt, value) => {
                info!("Received AppMsg::EditOpt");
                if self.conf.get(&opt).is_none() && value.is_empty() {
                    self.editedopts.remove(&opt);
                } else {
                    self.editedopts.insert(opt, value);
                }
            }
            AppMsg::Rebuild => {
                info!("Received AppMsg::Rebuild");
                let conf = match config::editconfigpath(
                    self.config.systemconfig.as_ref().unwrap(),
                    self.editedopts.clone(),
                ) {
                    Ok(x) => x,
                    Err(e) => {
                        self.rebuild.emit(RebuildMsg::FinishError(Some(format!(
                            "Error modifying configuration file.\n{}",
                            e
                        ))));
                        return;
                    }
                };
                self.rebuild.emit(RebuildMsg::Rebuild(
                    conf,
                    self.config.systemconfig.as_ref().unwrap().to_string(),
                    self.config.flake.clone().map(|x| {
                        if let Some(flakearg) = &self.config.flakearg {
                            format!("{}#{}", x, flakearg)
                        } else {
                            x
                        }
                    }),
                ));
            }
            AppMsg::ResetConfig => {
                info!("Received AppMsg::ResetConfig");
                self.update_editedopts(|x| x.clear());
                if self.page == Page::Option {
                    sender.input(AppMsg::OpenOption(
                        self.position.clone(),
                        self.refposition.clone(),
                    ));
                }
            }
            AppMsg::SaveConfig => {
                info!("Received AppMsg::SaveConfig");
                self.update_editedopts(|x| x.clear());
                let conf = match parseconfig(self.config.systemconfig.as_ref().unwrap()) {
                    Ok(x) => x,
                    Err(_) => {
                        sender.input(AppMsg::LoadError(
                            String::from("Error loading configuration file"),
                            format!(
                                "<tt>{}</tt> may be an invalid configuration file",
                                self.config.systemconfig.as_ref().unwrap()
                            ),
                        ));
                        return;
                    }
                };
                self.set_conf(conf);
                sender.input(AppMsg::SetBusy(true));
                self.set_page(Page::Loading);
                sender.input(AppMsg::TryLoad);
            }
            AppMsg::ShowPrefMenu => {
                info!("Received AppMsg::ShowPrefMenu");
                self.preferencespage
                    .emit(PreferencesPageMsg::Show(self.config.clone()));
            }
            AppMsg::ShowPrefMenuErr => {
                info!("Received AppMsg::ShowPrefMenuErr");
                self.preferencespage
                    .emit(PreferencesPageMsg::ShowErr(self.config.clone()));
            }
            AppMsg::SetDarkMode(dark) => {
                info!("Received AppMsg::SetDarkMode");
                let scheme = if dark { "Adwaita-dark" } else { "Adwaita" };
                self.optionpage
                    .emit(OptPageMsg::SetScheme(scheme.to_string()));
                self.saveerror
                    .emit(SaveErrorMsg::SetScheme(scheme.to_string()));
                self.rebuild.emit(RebuildMsg::SetScheme(scheme.to_string()));
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(scheme));
            }
            AppMsg::AddAttr => {
                info!("Received AppMsg::AddAttr");
                let attributes_guard = self.attributes.guard();
                let mut attrvec = vec![];
                for i in 0..attributes_guard.len() {
                    attrvec.push(attributes_guard.get(i).unwrap().clone());
                }
                attributes_guard.drop();

                match self.nameorstar {
                    AddAttrOptions::Name => self.nameentry.emit(NameEntryMsg::Show(
                        self.position.join("."),
                        attrvec
                            .iter()
                            .map(|x| x.value.last().unwrap_or(&String::default()).to_string())
                            .collect::<Vec<_>>(),
                    )),
                    AddAttrOptions::Star => {
                        let pos = self.position.join(".");
                        self.update_starattrs(|x| {
                            x.insert(pos.to_string(), *x.get(&pos).unwrap_or(&0) + 1);
                        });
                        sender.input(AppMsg::MoveToSelf);
                    }
                    AddAttrOptions::None => {
                        error!("Cannot add attribute without name or star");
                    }
                }
            }
            AppMsg::AddNameAttr(position, name) => {
                info!("Received AppMsg::AddNameAttr");
                let pos = if let Some(x) = position {
                    x
                } else {
                    self.position.join(".")
                };
                self.update_nameattrs(|x| {
                    if let Some(v) = x.get(&pos) {
                        if !v.contains(&name) {
                            let mut v = v.clone();
                            v.push(name.to_string());
                            x.insert(pos.to_string(), v);
                        }
                    } else {
                        let v = vec![name.to_string()];
                        x.insert(pos.to_string(), v);
                    }
                });
                debug!("ADD NEW <NAME> {:?}", self.nameattrs);
                sender.input(AppMsg::MoveToSelf);
            }
            AppMsg::AddStar(pos) => {
                info!("Received AppMsg::AddStar");
                self.update_starattrs(|x| {
                    x.insert(pos.to_string(), *x.get(&pos).unwrap_or(&0) + 1);
                });
            }
            AppMsg::OpenSearchOption(pos, refpos) => {
                info!("Received AppMsg::OpenSearchOption");
                self.searchpage
                    .emit(SearchPageMsg::OpenOption(pos, Some(refpos)));
            }
            AppMsg::SaveQuit => {
                info!("Received AppMsg::SaveQuit");
                let conf = match config::editconfigpath(
                    self.config.systemconfig.as_ref().unwrap(),
                    self.editedopts.clone(),
                ) {
                    Ok(x) => x,
                    Err(e) => {
                        self.rebuild.emit(RebuildMsg::FinishError(Some(format!(
                            "Error modifying configuration file.\n{}",
                            e
                        ))));
                        return;
                    }
                };
                self.rebuild.emit(RebuildMsg::WriteConfigQuit(
                    conf,
                    self.config.systemconfig.as_ref().unwrap().to_string(),
                ));
                self.editedopts.clear();
            }
            AppMsg::ShowAboutPage => {
                let about = AboutPageModel::builder()
                    .launch(self.mainwindow.clone().upcast())
                    .forward(sender.input_sender(), identity);
                about.widget().show();
            }
            AppMsg::SetModifiedOnly(modified) => {
                info!("Received AppMsg::SetModifiedOnly({})", modified);
                self.set_modifiedonly(modified);
                self.searchpage
                    .emit(SearchPageMsg::SetModifiedOnly(modified, self.search));
                if !self.search && self.page == Page::List {
                    sender.input(AppMsg::MoveToSelf)
                }
            }
            _ => {}
        }
    }
}

relm4::new_action_group!(MenuActionGroup, "menu");
relm4::new_stateless_action!(PreferencesAction, MenuActionGroup, "preferences");
relm4::new_stateless_action!(AboutAction, MenuActionGroup, "about");
relm4::new_stateful_action!(ModifiedAction, MenuActionGroup, "modified", u8, u8);

relm4::new_action_group!(WindowActionGroup, "window");
relm4::new_stateless_action!(SearchAction, WindowActionGroup, "search");
