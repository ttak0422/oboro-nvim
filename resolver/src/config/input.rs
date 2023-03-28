use serde::Deserialize;

/// `start` plugin.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StartPlugin {
    pub id: String,
    pub plugin: String,
    /// lua code execute at startup.
    pub startup: String,
}

/// `opt` plugin.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OptPlugin {
    pub id: String,
    pub plugin: String,
    /// lua code execute at startup.
    pub startup: String,
    /// lua code execute on load.
    pub config: String,
    /// plugin dependencies.
    pub deps: Vec<String>,
    /// bundle dependencies.
    pub dep_bundles: Vec<String>,
    /// load on modules.
    pub mods: Vec<String>,
    /// load on events.
    pub evs: Vec<String>,
    /// load on filetypes.
    pub fts: Vec<String>,
    /// load on commands.
    pub cmds: Vec<String>,
    /// load lazy.
    pub lazy: bool,
}

/// plugin bundle.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Bundle {
    pub id: String,
    pub plugins: Vec<String>,
    /// lua code execute at startup.
    pub startup: String,
    /// lua code execute on load.
    pub config: String,
    /// plugin dependencies.
    pub deps: Vec<String>,
    /// bundle dependencies.
    pub dep_bundles: Vec<String>,
    /// load on modules.
    pub mods: Vec<String>,
    /// load on events.
    pub evs: Vec<String>,
    /// load on filetypes.
    pub fts: Vec<String>,
    /// load on commands.
    pub cmds: Vec<String>,
    /// load lazy.
    pub lazy: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OboroPluginConfig {
    pub start_plugins: Vec<StartPlugin>,
    pub opt_plugins: Vec<OptPlugin>,
    pub bundles: Vec<Bundle>,
}
