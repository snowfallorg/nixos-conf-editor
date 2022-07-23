use super::about::AboutModel;
use super::nameentry::NameEntryModel;
use super::optionpage::*;
use super::preferencespage::PrefModel;
use super::preferencespage::WelcomeModel;
use super::preferencespage::WelcomeMsg;
use super::rebuild::RebuildModel;
use super::savechecking::SaveErrorModel;
use super::savechecking::SaveErrorMsg;
use super::searchentry::SearchEntryModel;
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
use crate::parse::{
    config::{opconfigured, parseconfig},
    options::*,
};
use crate::ui::about::AboutMsg;
use crate::ui::nameentry::NameEntryMsg;
use crate::ui::preferencespage::PrefMsg;
use crate::ui::rebuild::RebuildMsg;
use crate::ui::searchentry::SearchEntryMsg;
use crate::ui::windowloading::LoadErrorMsg;
use adw::prelude::*;
use log::*;
use relm4::{actions::*, factory::*, AppUpdate, Model, RelmApp, Sender, Widgets, *};
use std::collections::HashMap;

#[tracker::track]
pub struct AppModel {
    pub position: Vec<String>,
    pub refposition: Vec<String>,
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
    nameattrs: HashMap<String, Vec<String>>,
    starattrs: HashMap<String, usize>,
    pub configpath: String,
    pub scheme: Option<sourceview5::StyleScheme>,
    fieldreplace: HashMap<usize, String>,
    nameorstar: AddAttrOptions,
    flake: Option<String>,
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

pub enum AppMsg {
    Welcome,
    InitialLoad(LoadValues),
    LoadError(String, String),
    TryLoad,
    Close,
    SetConfPath(String, Option<String>),
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
    SetDarkMode(bool),
    AddAttr,
    AddNameAttr(Option<String>, String),
    AddStar(String),
    OpenSearchOption(Vec<String>, Vec<String>),
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
    nameentry: RelmComponent<NameEntryModel, AppModel>,
    searchpageentry: RelmComponent<SearchEntryModel, AppModel>,
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
                info!("Received AppMsg::Welcome");
                send!(components.welcome.sender(), WelcomeMsg::Show);
            }
            AppMsg::InitialLoad(x) => {
                info!("Received AppMsg::InitialLoad");
                self.set_data(x.data);
                self.set_tree(x.tree);
                self.set_conf(x.conf);
                trace!("CONF:\n{:#?}", self.conf);
                self.update_position(|x| x.clear());
                let options = self
                    .data
                    .iter()
                    .map(|(k, data)| {
                        let mut v = k.split('.').map(|x| x.to_string()).collect::<Vec<_>>();
                        let attr = v.pop().unwrap_or_default();
                        (k.to_string(), opconfigured(&self.conf, &v, attr), data.description.to_string())
                    })
                    .collect::<Vec<_>>();
                send!(
                    components.searchpage.sender(),
                    SearchPageMsg::LoadOptions(options)
                );
                self.set_busy(false);
                send!(sender, AppMsg::MoveTo(vec![], vec![]));
            }
            AppMsg::LoadError(s, s2) => {
                info!("Received AppMsg::LoadError");
                self.set_busy(false);
                send!(components.loaderror.sender(), LoadErrorMsg::Show(s, s2));
            }
            AppMsg::TryLoad => {
                info!("Received AppMsg::TryLoad");
                self.set_busy(true);
                components
                    .windowloading
                    .sender()
                    .blocking_send(WindowAsyncHandlerMsg::RunWindow(
                        self.configpath.to_string(),
                    ))
                    .unwrap();
            }
            AppMsg::Close => {
                info!("Received AppMsg::Close");
                relm4::gtk_application().quit();
            }
            AppMsg::SetConfPath(s, b) => {
                info!("Received AppMsg::SetConfPath");
                self.set_busy(true);
                self.set_page(Page::Loading);
                self.set_configpath(s.clone());
                self.set_flake(b.clone());
                components
                    .windowloading
                    .sender()
                    .blocking_send(WindowAsyncHandlerMsg::SetConfig(s, b))
                    .unwrap();
            }
            AppMsg::MoveToSelf => {
                info!("Received AppMsg::MoveToSelf");
                send!(sender, AppMsg::MoveTo(self.position.clone(), self.refposition.clone()));
            }
            AppMsg::MoveToRow(pos) => {
                info!("Received AppMsg::MoveToRow");
                match self.attributes.iter().find(|x| x.value == pos) {
                    Some(x) => {
                        debug!("FOUND ATTR: {:?}", x);
                        send!(sender, AppMsg::MoveTo(x.value.to_vec(), x.refvalue.to_vec()));
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
                if let Some(x) = self.attributes.iter().find(|x| x.value.eq(&pos)) {
                    if let Some(y) = &x.replacefor {
                        p.pop();
                        p.push(String::from(y));
                    }
                }
                // let mut newref = self.refposition.clone();
                // if pos.len() < self.refposition.len() {
                //     newref = self.refposition[0..pos.len()].to_vec();
                // } else {
                //     newref.append(&mut p[self.refposition.len()..].to_vec());
                // }

                debug!("NEW REFPOSITON: {:?}", newref);

                match attrloc(&self.tree, newref.to_vec()) {
                    Some(x) => {
                        let mut sortedoptions = x.options.clone();
                        sortedoptions.sort();
                        self.options.clear();
                        for op in sortedoptions {
                            let mut o = pos.to_vec();
                            let mut r = newref.to_vec();
                            o.push(op.to_string());
                            r.push(op.to_string());
                            self.options.push(OptPos {
                                value: o,
                                refvalue: r,
                                configured: opconfigured2(
                                    &self.configpath,
                                    &pos,
                                    &newref,
                                    op.clone(),
                                ),
                                modified: opconfigured(&self.editedopts, &pos, op),
                            });
                        }
                        self.attributes.clear();
                        let mut attributes = FactoryVec::new();
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
                                            modified: opconfigured(&self.editedopts, &pos, a.to_string()),
                                            replacefor: Some(String::from("<name>")),
                                        })
                                    }
                                }
                            } else if attr == "*" {
                                debug!("FOUND * ATTR");
                                hasnameorstar = AddAttrOptions::Star;
                                let v = getarrvals(&self.configpath, &pos);
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
                                        modified: opconfigured(
                                            &self.editedopts,
                                            &pos,
                                            i.to_string(),
                                        ),
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
                                        modified: opconfigured(
                                            &self.editedopts,
                                            &pos,
                                            i.to_string(),
                                        ),
                                        replacefor: Some(String::from("*")),
                                    })
                                }
                            } else {
                                let mut p = pos.to_vec();
                                let mut r = newref.to_vec();
                                p.push(attr.to_string());
                                r.push(attr.to_string());
                                attributes.push(AttrPos {
                                    value: p,
                                    refvalue: r,
                                    configured: opconfigured2(
                                        &self.configpath,
                                        &pos,
                                        &newref,
                                        attr.to_string(),
                                    ),
                                    modified: opconfigured(
                                        &self.editedopts,
                                        &newref,
                                        attr.to_string(),
                                    ),
                                    replacefor: None,
                                });
                            }
                        }
                        if !pos.is_empty() {
                            self.posbtn.clear();
                            let mut pref = vec![];
                            let mut rref = vec![];
                            for i in 0..pos.len() {
                                pref.push(pos[i].clone());
                                rref.push(newref[i].clone());
                                self.posbtn.push(AttrBtn {
                                    value: pref.to_vec(),
                                    refvalue: rref.to_vec(),
                                    opt: false,
                                });
                            }
                        }
                        let mut x = attributes.into_vec();
                        x.sort_by(|x, y| x.value.cmp(&y.value));
                        for attr in x {
                            self.attributes.push(attr);
                        }
                        debug!("Setting HNOS {:?}", hasnameorstar);
                        self.set_nameorstar(hasnameorstar);
                        self.set_position(pos);
                        self.set_refposition(newref);
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
            AppMsg::OpenOption(pos, newref) if !self.busy => {
                info!("Received AppMsg::OpenOption");
                // let mut newref = self.refposition.clone();
                // newref.append(&mut pos[self.refposition.len()..].to_vec());
                trace!("NEW REFPOSITON: {:?}", newref);
                let d = match self.data.get(&newref.join(".")) {
                    Some(x) => x,
                    None => {
                        error!("No data for {:?}", newref);
                        return true;
                    }
                };

                let conf = if let Some(x) = self.editedopts.get(&pos.join(".")) {
                    trace!("EDITED");
                    x.to_string()
                } else if let Some(n) = self.conf.get(&pos.join(".")) {
                    trace!("CONFIGURED");
                    n.to_string()
                } else if let Ok(v) = readval(&self.configpath, &pos.join("."), &newref.join(".")) {
                    trace!("READ");
                    v
                } else {
                    trace!("EMPTY");
                    String::default()
                };
                components
                    .optionpage
                    .send(OptPageMsg::UpdateOption(
                        Box::new(d.clone()),
                        pos.to_vec(),
                        newref.to_vec(),
                        conf,
                        self.data.keys().map(|x| x.to_string()).collect::<Vec<String>>(),
                    ))
                    .unwrap();
                self.options.clear();
                self.attributes.clear();
                self.posbtn.clear();
                let mut pref = vec![];
                let mut rref = vec![];
                let mut p2 = pos.clone();
                let mut r2 = newref.clone();
                p2.pop();
                r2.pop();
                for i in 0..p2.len() {
                    pref.push(p2[i].clone());
                    rref.push(r2[i].clone());
                    self.posbtn.push(AttrBtn {
                        value: pref.to_vec(),
                        refvalue: rref.to_vec(),
                        opt: false,
                    });
                }
                self.posbtn.push(AttrBtn {
                    value: pos.to_vec(),
                    refvalue: newref.to_vec(),
                    opt: true,
                });
                self.set_position(pos);
                self.set_refposition(newref);
                self.set_header(HeaderBar::List);
                self.set_search(false);
                self.set_page(Page::Option);
            }
            AppMsg::OpenOptionRow(pos) => {
                info!("Received AppMsg::OpenOptionRow");
                match self.options.iter().find(|x| x.value == pos) {
                    Some(x) => {
                        send!(sender, AppMsg::OpenOption(x.value.to_vec(), x.refvalue.to_vec()));
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
                    send!(sender, AppMsg::HideSearch);
                } else {
                    send!(sender, AppMsg::ShowSearch);
                }
            }
            AppMsg::ShowSearchPage(s) if !self.busy => {
                info!("Received AppMsg::ShowSearchPage");
                components
                    .searchpage
                    .send(SearchPageMsg::Search(s))
                    .unwrap();
                self.set_search(true)
            }
            AppMsg::HideSearchPage => {
                info!("Received AppMsg::HideSearchPage");
                self.set_search(false)
            }
            AppMsg::ShowSearchPageEntry(pos) => {
                info!("Received AppMsg::ShowSearchPageEntry");
                // Input a string of the form "service.<name>.groups.*.uid" and return a vector of all possible existing options for that string.
                fn getposdata(pos: &Vec<String>, conf: &HashMap<String, String>, nameattrs: &HashMap<String, Vec<String>>, starattrs: &HashMap<String, usize>, configpath: &str) -> Vec<Vec<String>> {
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
                                out.append(&mut getposdata(&newpos, conf, nameattrs, starattrs, configpath));
                            }
                            return out;
                        } else if pos[i] == "*" {
                            let v = getarrvals(configpath, &pos[..i].to_vec());
                            let mut n = v.len();
                            if let Some(x) = starattrs.get(&pos[..i].join(".")) {
                                n += *x;
                            }
                            let mut out = vec![];
                            for j in 0..n {
                                let mut newpos = pos.clone();
                                newpos[i] = j.to_string();
                                out.append(&mut getposdata(&newpos, conf, nameattrs, starattrs, configpath));
                            }
                            return out;
                        }
                    }
                    return vec![pos.to_vec()];
                }

                let data = getposdata(&pos, &self.conf, &self.nameattrs, &self.starattrs, &self.configpath).iter().map(|x| x.join(".")).collect::<Vec<String>>();
                send!(components.searchpageentry.sender(), SearchEntryMsg::Show(pos, data));
            }
            AppMsg::SetBusy(b) => {
                info!("Received AppMsg::SetBusy");
                self.set_busy(b)
            }
            AppMsg::SaveError(msg) => {
                info!("Received AppMsg::SaveError");
                components.saveerror.send(SaveErrorMsg::Show(msg)).unwrap()
            }
            AppMsg::SaveWithError => {
                info!("Received AppMsg::SaveWithError");
                components
                    .optionpage
                    .send(OptPageMsg::DoneSaving(true, "true\n".to_string()))
                    .unwrap()
            }
            AppMsg::SaveErrorReset => {
                info!("Received AppMsg::SaveErrorReset");
                components.optionpage.send(OptPageMsg::ResetConf).unwrap()
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
                let conf = match config::editconfigpath(&self.configpath, self.editedopts.clone()) {
                    Ok(x) => x,
                    Err(e) => {
                        components
                            .rebuild
                            .send(RebuildMsg::FinishError(Some(format!(
                                "Error modifying configuration file.\n{}",
                                e
                            ))))
                            .unwrap();
                        return true;
                    }
                };
                send!(
                    components.rebuild.sender(),
                    RebuildMsg::Rebuild(conf, self.configpath.to_string(), self.flake.clone())
                );
            }
            AppMsg::ResetConfig => {
                info!("Received AppMsg::ResetConfig");
                self.update_editedopts(|x| x.clear());
                if self.page == Page::Option {
                    send!(sender, AppMsg::OpenOption(self.position.clone(), self.refposition.clone()));
                }
            }
            AppMsg::SaveConfig => {
                info!("Received AppMsg::SaveConfig");
                self.update_editedopts(|x| x.clear());
                let conf = match parseconfig(&self.configpath) {
                    Ok(x) => x,
                    Err(_) => {
                        send!(
                            sender,
                            AppMsg::LoadError(
                                String::from("Error loading configuration file"),
                                format!(
                                    "<tt>{}</tt> may be an invalid configuration file",
                                    self.configpath
                                )
                            )
                        );
                        return true;
                    }
                };
                self.set_conf(conf);
                send!(sender, AppMsg::SetBusy(true));
                self.set_page(Page::Loading);
                send!(sender, AppMsg::TryLoad);
            }
            AppMsg::ShowPrefMenu => {
                info!("Received AppMsg::ShowPrefMenu");
                send!(
                    components.preferences.sender(),
                    PrefMsg::Show(self.configpath.to_string(), self.flake.clone())
                );
            }
            AppMsg::SetDarkMode(dark) => {
                info!("Received AppMsg::SetDarkMode");
                let scheme = if dark { "Adwaita-dark" } else { "Adwaita" };
                send!(
                    components.optionpage.sender(),
                    OptPageMsg::SetScheme(scheme.to_string())
                );
                send!(
                    components.saveerror.sender(),
                    SaveErrorMsg::SetScheme(scheme.to_string())
                );
                send!(
                    components.rebuild.sender(),
                    RebuildMsg::SetScheme(scheme.to_string())
                );
                self.set_scheme(sourceview5::StyleSchemeManager::default().scheme(scheme));
            }
            AppMsg::AddAttr => {
                info!("Received AppMsg::AddAttr");
                match self.nameorstar {
                    AddAttrOptions::Name => {
                        components
                            .nameentry
                            .send(NameEntryMsg::Show(
                                self.position.join("."),
                                self.attributes
                                    .iter()
                                    .map(|x| {
                                        x.value.last().unwrap_or(&String::default()).to_string()
                                    })
                                    .collect::<Vec<_>>(),
                            ))
                            .unwrap();
                    }
                    AddAttrOptions::Star => {
                        let pos = self.position.join(".");
                        self.update_starattrs(|x| {
                            x.insert(pos.to_string(), *x.get(&pos).unwrap_or(&0) + 1);
                        });
                        send!(sender, AppMsg::MoveToSelf);
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
                        // let confvals = getconfvals(&conf, &pos.split('.').map(|x| x.to_string()).collect::<Vec<_>>());
                        if !v.contains(&name) { // && !confvals.contains(&name) {
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
                send!(sender, AppMsg::MoveToSelf);
            }
            AppMsg::AddStar(pos) => {
                info!("Received AppMsg::AddStar");
                self.update_starattrs(|x| {
                    x.insert(pos.to_string(), *x.get(&pos).unwrap_or(&0) + 1);
                });
            }
            AppMsg::OpenSearchOption(pos, refpos) => {
                info!("Received AppMsg::OpenSearchOption");
                send!(components.searchpage.sender(), SearchPageMsg::OpenOption(pos, Some(refpos)));
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
            set_default_height: 650,
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
                                    send!(sender, AppMsg::MoveTo(vec![], vec![]))
                                },
                            },
                            factory!(model.posbtn),
                        },
                        add_child: search = &gtk::SearchEntry {
                                add_css_class: "inline",
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
                        add: attrgroup = &adw::PreferencesGroup {
                            set_title: "Attributes",
                            set_visible: track!(model.changed(AppModel::position()), !model.attributes.is_empty() || model.nameorstar != AddAttrOptions::None),
                            add = &gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_selection_mode: gtk::SelectionMode::None,
                                append: addrow = &adw::PreferencesRow { // Change to suffix once libadwaita-rs 0.2 is out
                                    set_visible: track!(model.changed(AppModel::nameorstar()), model.nameorstar != AddAttrOptions::None),
                                    set_title: "<ADD>",
                                    set_child = Some(&gtk::Box) {
                                        set_margin_all: 15,
                                        append = &gtk::Image {
                                            set_halign: gtk::Align::Center,
                                            set_hexpand: true,
                                            set_icon_name: Some("list-add-symbolic"),
                                            add_css_class: "accent",
                                            //set_label: "<b>+</b>",
                                            //set_use_markup: true,
                                            //add_css_class: "title-2",
                                        }
                                    }
                                },
                                factory!(model.attributes),
                                connect_row_activated(sender) => move |_, y| {
                                    if let Ok(l) = y.clone().downcast::<adw::PreferencesRow>() {
                                        if l.title() != "<ADD>" {
                                            let text = l.title().to_string();
                                            let v = text.split('.').map(|x| x.to_string()).collect::<Vec<String>>();
                                            send!(sender, AppMsg::MoveToRow(v));
                                        } else {
                                            send!(sender, AppMsg::AddAttr);
                                        }
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
                                        send!(sender, AppMsg::OpenOptionRow(v))
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
        components
            .windowloading
            .sender()
            .blocking_send(WindowAsyncHandlerMsg::GetConfigPath)
            .unwrap();
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
            adw::StyleManager::default()
                .connect_dark_notify(move |x| send!(sender, AppMsg::SetDarkMode(x.is_dark())));
        }
        send!(
            sender,
            AppMsg::SetDarkMode(adw::StyleManager::default().is_dark())
        );
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
        refposition: vec![],
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
        nameattrs: HashMap::new(),
        starattrs: HashMap::new(),
        configpath: String::from("/etc/nixos/configuration.nix"),
        flake: None,
        scheme: None,
        fieldreplace: HashMap::new(),
        nameorstar: AddAttrOptions::None,
        tracker: 0,
    };
    let app = RelmApp::with_app(model, adw::Application::new(
        Some(crate::config::APP_ID),
        adw::gio::ApplicationFlags::empty(),
    ));
    app.run();
}
