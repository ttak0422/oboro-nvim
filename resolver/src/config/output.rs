use std::collections::HashMap;

/// plugin loaded when startup vim.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct StartupPlugin<'a> {
    pub id: &'a str,
    pub plugin: &'a str,
    pub startup: &'a str,
}

/// plugin loaded on demand.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct LazyPlugin<'a> {
    pub id: &'a str,
    pub plugin: &'a str,
    pub startup: &'a str,
    pub config: &'a str,
    pub deps: Vec<&'a str>,
    pub dep_bundles: Vec<&'a str>,
}

/// plugins bundle.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bundle<'a> {
    pub id: &'a str,
    pub plugins: Vec<&'a str>,
    pub startup: &'a str,
    pub config: &'a str,
    pub deps: Vec<&'a str>,
    pub dep_bundles: Vec<&'a str>,
}

/// oboro config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OboroConfig<'a> {
    pub startup_plugins: Vec<StartupPlugin<'a>>,
    pub lazy_plugins: Vec<LazyPlugin<'a>>,
    pub bundles: Vec<Bundle<'a>>,
    pub mods: Vec<&'a str>,
    pub evs: Vec<&'a str>,
    pub fts: Vec<&'a str>,
    pub cmds: Vec<&'a str>,
    pub mod_map: HashMap<&'a str, Vec<&'a str>>,
    pub ev_map: HashMap<&'a str, Vec<&'a str>>,
    pub ft_map: HashMap<&'a str, Vec<&'a str>>,
    pub cmd_map: HashMap<&'a str, Vec<&'a str>>,
    pub lazys: Vec<&'a str>,
}
