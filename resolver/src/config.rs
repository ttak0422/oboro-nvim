pub mod input;
pub mod output;

use crate::config::input::OboroPluginConfig;
use crate::config::output::{Bundle, LazyPlugin, OboroConfig, StartupPlugin};
use anyhow::{bail, ensure, Result};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

trait Mergeable
where
    Self: std::default::Default + std::cmp::Eq,
{
    fn modified(&self) -> bool {
        self != &Default::default()
    }
    fn merge_into<'a>(&'a mut self, other: &'a mut Self) -> Result<()>;
}

trait Derivable<T, Key>
where
    T: Mergeable,
    Key: Hash + Eq,
{
    fn key(&self) -> Key;
}

impl Mergeable for bool {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        *self = *self || *other;
        Ok(())
    }
}

impl Mergeable for &str {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        let mod_self = self.modified();
        let mod_other = other.modified();
        if (mod_self && mod_other) && (self != other) {
            bail!("Conflicted `{}`, `{}`.", self, other)
        } else if mod_other {
            std::mem::swap(self, other)
        }
        Ok(())
    }
}

impl Mergeable for Vec<&str> {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        let mod_self = self.modified();
        let mod_other = other.modified();
        if mod_self && mod_other {
            bail!("Conflicted vector.")
        } else if mod_other {
            std::mem::swap(self, other)
        }
        Ok(())
    }
}

impl Mergeable for StartupPlugin<'_> {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        ensure!(
            self == &Default::default() || self.id == other.id,
            "Invalid merge ({}, {}).",
            &self.id,
            &other.id
        );
        self.id.merge_into(&mut other.id)?;
        self.plugin.merge_into(&mut other.plugin)?;
        self.startup.merge_into(&mut other.startup)?;
        Ok(())
    }
}

impl Mergeable for LazyPlugin<'_> {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        ensure!(
            self == &Default::default() || self.id == other.id,
            "Invalid merge ({}, {}).",
            &self.id,
            &other.id
        );
        self.id.merge_into(&mut other.id)?;
        self.plugin.merge_into(&mut other.plugin)?;
        self.startup.merge_into(&mut other.startup)?;
        self.config.merge_into(&mut other.config)?;
        self.deps.merge_into(&mut other.deps)?;
        self.dep_bundles.merge_into(&mut other.dep_bundles)?;
        Ok(())
    }
}

impl Mergeable for Bundle<'_> {
    fn merge_into(&mut self, other: &mut Self) -> Result<()> {
        ensure!(
            self == &Default::default() || self.id == other.id,
            "Invalid merge ({}, {}).",
            &self.id,
            &other.id
        );
        self.id.merge_into(&mut other.id)?;
        self.plugins.merge_into(&mut other.plugins)?;
        self.startup.merge_into(&mut other.startup)?;
        self.config.merge_into(&mut other.config)?;
        self.deps.merge_into(&mut other.deps)?;
        self.dep_bundles.merge_into(&mut other.dep_bundles)?;
        Ok(())
    }
}

impl<'a> Derivable<StartupPlugin<'a>, &'a str> for StartupPlugin<'a> {
    fn key(&self) -> &'a str {
        self.id
    }
}

impl<'a> Derivable<LazyPlugin<'a>, &'a str> for LazyPlugin<'a> {
    fn key(&self) -> &'a str {
        self.id
    }
}

impl<'a> Derivable<Bundle<'a>, &'a str> for Bundle<'a> {
    fn key(&self) -> &'a str {
        self.id
    }
}

fn derive<T, Key>(xs: Vec<T>) -> Result<Vec<T>>
where
    T: Mergeable + Derivable<T, Key>,
    Key: Hash + Eq,
{
    xs.into_iter()
        .into_group_map_by(|x| x.key())
        .into_values()
        .map(|v| {
            let def: T = Default::default();
            v.into_iter().try_fold(def, |mut acc, mut x| {
                acc.merge_into(&mut x)?;
                Ok(acc)
            })
        })
        .collect()
}

/// to Vec<&str>.
fn to_str_vector(v: &[String]) -> Vec<&str> {
    v.iter().map(|x| x.as_str()).collect()
}

/// dedup vector.
fn to_unique_vector<T: Hash + Eq>(v: Vec<T>) -> Vec<T> {
    v.into_iter().collect::<HashSet<_>>().into_iter().collect()
}

/// dedup values.
fn to_unique_map<T: Hash + Eq>(m: HashMap<&str, Vec<T>>) -> HashMap<&str, Vec<T>> {
    m.into_iter()
        .map(|(k, v)| (k, to_unique_vector(v)))
        .collect::<HashMap<&str, Vec<T>>>()
}

/// just mapping.
fn map(config: &OboroPluginConfig) -> OboroConfig<'_> {
    // import all
    let mut startup_plugins: Vec<StartupPlugin<'_>> = Vec::new();
    let mut lazy_plugins = Vec::new();
    let mut bundles = Vec::new();

    let mut mod_map = HashMap::<&str, Vec<&str>>::new();
    let mut ev_map = HashMap::<&str, Vec<&str>>::new();
    let mut ft_map = HashMap::<&str, Vec<&str>>::new();
    let mut cmd_map = HashMap::<&str, Vec<&str>>::new();
    let mut lazys = Vec::new();

    // start
    for plugin in config.start_plugins.iter() {
        println!("map start plugin: {}", &plugin.id);
        startup_plugins.push(StartupPlugin {
            id: &plugin.id,
            plugin: &plugin.plugin,
            startup: &plugin.startup,
        });
    }

    // opt
    for plugin in config.opt_plugins.iter() {
        println!("map opt plugin: {}", &plugin.id);
        lazy_plugins.push(LazyPlugin {
            id: &plugin.id,
            plugin: &plugin.plugin,
            startup: &plugin.startup,
            config: &plugin.config,
            deps: to_str_vector(&plugin.deps),
            dep_bundles: to_str_vector(&plugin.dep_bundles),
        });
        for module in to_str_vector(&plugin.mods) {
            mod_map.entry(module).or_insert(Vec::new()).push(&plugin.id);
        }
        for ev in to_str_vector(&plugin.evs) {
            ev_map.entry(ev).or_insert(Vec::new()).push(&plugin.id);
        }
        for ft in to_str_vector(&plugin.fts) {
            ft_map.entry(ft).or_insert(Vec::new()).push(&plugin.id);
        }
        for cmd in to_str_vector(&plugin.cmds) {
            cmd_map.entry(cmd).or_insert(Vec::new()).push(&plugin.id);
        }
        if plugin.lazy {
            lazys.push(plugin.id.as_str());
        }
    }

    // bundle
    for bundle in config.bundles.iter() {
        println!("map bundle: {}", &bundle.id);
        bundles.push(Bundle {
            id: &bundle.id,
            plugins: to_str_vector(&bundle.plugins),
            startup: &bundle.startup,
            config: &bundle.config,
            deps: to_str_vector(&bundle.deps),
            dep_bundles: to_str_vector(&bundle.dep_bundles),
        });
        for module in to_str_vector(&bundle.mods) {
            mod_map.entry(module).or_insert(Vec::new()).push(&bundle.id);
        }
        for ev in to_str_vector(&bundle.evs) {
            ev_map.entry(ev).or_insert(Vec::new()).push(&bundle.id);
        }
        for ft in to_str_vector(&bundle.fts) {
            ft_map.entry(ft).or_insert(Vec::new()).push(&bundle.id);
        }
        for cmd in to_str_vector(&bundle.cmds) {
            cmd_map.entry(cmd).or_insert(Vec::new()).push(&bundle.id);
        }
        if bundle.lazy {
            lazys.push(bundle.id.as_str());
        }
    }

    OboroConfig {
        startup_plugins,
        lazy_plugins,
        bundles,
        mod_map,
        ev_map,
        ft_map,
        cmd_map,
        lazys,
        ..Default::default()
    }
}

fn validate(config: &OboroConfig) -> Result<()> {
    // validate `id`.
    let start_id_set = config
        .startup_plugins
        .iter()
        .map(|x| x.id)
        .collect::<HashSet<_>>();
    let lazy_ids = config.lazy_plugins.iter().map(|x| x.id).collect::<Vec<_>>();
    for lazy_id in lazy_ids.iter() {
        if start_id_set.contains(*lazy_id) {
            bail!(
                "`{}` is configured in both `startPlugin` and `optPlugin` or `bundle`.",
                *lazy_id
            );
        }
    }
    let lazy_id_set = lazy_ids.into_iter().collect::<HashSet<_>>();
    let bundle_ids = config.bundles.iter().map(|x| x.id).collect::<Vec<_>>();
    for bundle_id in bundle_ids.iter() {
        if start_id_set.contains(*bundle_id) || lazy_id_set.contains(bundle_id) {
            bail!("the id `{}` is also used other plugin.", *bundle_id);
        }
    }
    Ok(())
}

/// resolve config.
pub fn resolve(config: &OboroPluginConfig) -> Result<OboroConfig<'_>> {
    // pub fn resolve<'a>(config: &'a OboroPluginConfig) -> Result<OboroConfig<'a>> {
    let cfg = map(config);

    validate(&cfg)?;

    // TODO: test
    let mod_map = to_unique_map(cfg.mod_map);
    let ev_map = to_unique_map(cfg.ev_map);
    let ft_map = to_unique_map(cfg.ft_map);
    let cmd_map = to_unique_map(cfg.cmd_map);
    let mods = mod_map.keys().cloned().collect();
    let evs = ev_map.keys().cloned().collect();
    let fts = ft_map.keys().cloned().collect();
    let cmds = cmd_map.keys().cloned().collect();

    Ok(OboroConfig {
        startup_plugins: derive(cfg.startup_plugins)?,
        lazy_plugins: derive(cfg.lazy_plugins)?,
        bundles: derive(cfg.bundles)?,
        mods,
        evs,
        fts,
        cmds,
        mod_map,
        ev_map,
        ft_map,
        cmd_map,
        lazys: to_unique_vector(cfg.lazys),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(arg, exp,
        case(vec![
                String::from("foo"),
                String::from("bar"),
                String::from("baz"),
            ], vec!["foo", "bar", "baz"]),
        case(vec![], vec![]),
    )]
    fn convert_string_vector(arg: Vec<String>, exp: Vec<&str>) {
        // act:
        let act = to_str_vector(&arg);

        // assert:
        assert_eq!(act, exp);
    }

    #[rstest(arg, exp,
        case(vec![1, 2, 2, 3, 3, 3], vec![1, 2, 3]),
        case(vec![], vec![]),
    )]
    fn dedup_vector(arg: Vec<i32>, exp: Vec<i32>) {
        // act:
        let act = to_unique_vector(arg);

        // assert:
        assert_eq!(act.len(), exp.len());
        for x in exp.iter() {
            assert!(act.contains(x));
        }
    }

    #[rstest(
        arg_x,
        arg_y,
        exp,
        case(true, true, true),
        case(false, true, true),
        case(true, false, true),
        case(false, false, false)
    )]
    fn merge_bool(mut arg_x: bool, mut arg_y: bool, exp: bool) {
        // act:
        arg_x.merge_into(&mut arg_y).unwrap();

        // assert:
        assert_eq!(arg_x, exp);
    }

    #[rstest(arg_x, arg_y, exp,
        case("foo", "foo", "foo"),
        case("", "foo", "foo"),
        case("foo", "", "foo"),
        case("", "", ""),
        #[should_panic]
        case("foo", "bar", "_"),
    )]
    fn merge_str(arg_x: &str, arg_y: &str, exp: &str) {
        // arrange:
        let mut x = arg_x;
        let mut y = arg_y;

        // act:
        x.merge_into(&mut y).unwrap();
        dbg!("{} {} {}", x, y, exp);

        // assert:
        assert_eq!(x, exp)
    }

    #[rstest(arg_x, arg_y, exp,
        case(vec!["a"], vec![], vec!["a"]),
        case(vec![], vec!["a"], vec!["a"]),
        case(vec![], vec![], vec![]),
        #[should_panic]
        case(vec!["a"], vec!["a"], vec![]),
    )]
    fn merge_vec(arg_x: Vec<&str>, arg_y: Vec<&str>, exp: Vec<&str>) {
        // arrange:
        let mut x = arg_x.clone();
        let mut y = arg_y.clone();

        // act:
        x.merge_into(&mut y).unwrap();

        // assert:
        assert_eq!(x, exp);
    }

    #[rstest(arg_x, arg_y, exp,
        case(
            StartupPlugin { id : "foo", plugin : "plugin", startup: "startup" },
            StartupPlugin { id : "foo", plugin : "", startup: "" },
            StartupPlugin { id : "foo", plugin : "plugin", startup: "startup" }
        ),
        case(
            StartupPlugin { id : "foo", plugin : "", startup: "" },
            StartupPlugin { id : "foo", plugin : "plugin", startup: "startup" },
            StartupPlugin { id : "foo", plugin : "plugin", startup: "startup" }
        ),
        #[should_panic]
        case(
            StartupPlugin { id : "foo", plugin : "", startup: "" },
            StartupPlugin { id : "bar", plugin : "", startup: "" },
            StartupPlugin { id : "_", plugin : "_", startup: "_" }
        ),
        #[should_panic]
        case(
            StartupPlugin { id : "foo", plugin : "p", startup: "startup1" },
            StartupPlugin { id : "foo", plugin : "p", startup: "startup2" },
            StartupPlugin { id : "_", plugin : "_", startup: "_" }
        ),
    )]
    fn merge_startup(arg_x: StartupPlugin, arg_y: StartupPlugin, exp: StartupPlugin) {
        // arrange:
        let mut x = arg_x.clone();
        let mut y = arg_y.clone();

        // act:
        x.merge_into(&mut y).unwrap();

        // assert:
        assert_eq!(x, exp);
    }

    #[rstest(arg_x, arg_y, exp,
        case(
            LazyPlugin { id : "foo", plugin : "plugin", startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            LazyPlugin { id : "foo", plugin : "",       startup: "",        config: "config", deps: vec![],      dep_bundles: vec![] },
            LazyPlugin { id : "foo", plugin : "plugin", startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
        ),
        case(
            LazyPlugin { id : "foo", plugin : "",       startup: "",        config: "config", deps: vec![],      dep_bundles: vec![] },
            LazyPlugin { id : "foo", plugin : "plugin", startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            LazyPlugin { id : "foo", plugin : "plugin", startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
        ),
        #[should_panic]
        case(
            LazyPlugin { id : "foo",  plugin : "plugin", startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            LazyPlugin { id : "hoge", plugin : "",       startup: "",        config: "",       deps: vec![],      dep_bundles: vec![] },
            LazyPlugin { id : "_",    plugin : "_",      startup: "_",       config: "_",      deps: vec![],      dep_bundles: vec![] }
        ),
        #[should_panic]
        case(
            LazyPlugin { id : "foo", plugin : "_", startup: "_", config: "_", deps: vec!["conflict_foo1"], dep_bundles: vec![] },
            LazyPlugin { id : "foo", plugin : "_", startup: "_", config: "_", deps: vec!["conflict_foo2"], dep_bundles: vec![] },
            LazyPlugin { id : "_",   plugin : "_", startup: "_", config: "_", deps: vec![],                dep_bundles: vec![] }
        ),
    )]
    fn merge_lazy(arg_x: LazyPlugin, arg_y: LazyPlugin, exp: LazyPlugin) {
        // arrange:
        let mut x = arg_x.clone();
        let mut y = arg_y.clone();

        // act:
        x.merge_into(&mut y).unwrap();

        // assert:
        assert_eq!(x, exp);
    }

    #[rstest(arg_x, arg_y, exp,
        case(
            Bundle { id : "foo", plugins : vec!["plugins"], startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            Bundle { id : "foo", plugins : vec![],          startup: "",        config: "config", deps: vec![],      dep_bundles: vec![] },
            Bundle { id : "foo", plugins : vec!["plugins"], startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
        ),
        case(
            Bundle { id : "foo", plugins : vec![],          startup: "",        config: "config", deps: vec![],      dep_bundles: vec![] },
            Bundle { id : "foo", plugins : vec!["plugins"], startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            Bundle { id : "foo", plugins : vec!["plugins"], startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
        ),
        #[should_panic]
        case(
            Bundle { id : "foo",  plugins : vec!["plugins"], startup: "startup", config: "config", deps: vec!["bar"], dep_bundles: vec!["baz"] },
            Bundle { id : "hoge", plugins : vec![],          startup: "",        config: "",       deps: vec![],      dep_bundles: vec![] },
            Bundle { id : "_",    plugins : vec![],          startup: "_",       config: "_",      deps: vec![],      dep_bundles: vec![] }
        ),
        #[should_panic]
        case(
            Bundle { id : "foo", plugins : vec![], startup: "_", config: "_", deps: vec!["conflict_foo1"], dep_bundles: vec![] },
            Bundle { id : "foo", plugins : vec![], startup: "_", config: "_", deps: vec!["conflict_foo2"], dep_bundles: vec![] },
            Bundle { id : "_",   plugins : vec![], startup: "_", config: "_", deps: vec![], dep_bundles: vec![] }
        ),
    )]
    fn merge_bundle(arg_x: Bundle, arg_y: Bundle, exp: Bundle) {
        // arrange:
        let mut x = arg_x.clone();
        let mut y = arg_y.clone();

        // act:
        x.merge_into(&mut y).unwrap();

        // assert:
        assert_eq!(x, exp);
    }

    #[rstest(arg, exp,
        case(
            vec![
                StartupPlugin { id : "foo", plugin : "", startup: "" },
                StartupPlugin { id : "bar", plugin : "", startup: "" },
                StartupPlugin { id : "foo", plugin : "foo plugin", startup: "foo startup" },
                StartupPlugin { id : "bar", plugin : "foo plugin", startup: "foo startup" },
            ],
            vec![
                StartupPlugin { id : "foo", plugin : "foo plugin", startup: "foo startup" },
                StartupPlugin { id : "bar", plugin : "foo plugin", startup: "foo startup" },
            ],
        ),
        case(vec![],vec![]),
    )]
    fn derive_startup(arg: Vec<StartupPlugin>, mut exp: Vec<StartupPlugin>) {
        // arrange:
        exp.sort();

        // act:
        let mut act = derive(arg).unwrap();
        act.sort();

        // assert:
        assert_eq!(act, exp);
    }

    #[rstest(arg, exp,
        case(
            vec![
                LazyPlugin { id : "foo", plugin : "", startup: "", config: "", deps: vec![], dep_bundles: vec![] },
                LazyPlugin { id : "bar", plugin : "", startup: "", config: "", deps: vec![], dep_bundles: vec![] },
                LazyPlugin { id : "foo", plugin : "foo plugin", startup: "foo startup", config: "foo config", deps: vec!["foo"], dep_bundles: vec!["foo_dep"] },
                LazyPlugin { id : "bar", plugin : "bar plugin", startup: "bar startup", config: "bar config", deps: vec!["foo"], dep_bundles: vec!["bar_dep"] },
            ],
            vec![
                LazyPlugin { id : "foo", plugin : "foo plugin", startup: "foo startup", config: "foo config", deps: vec!["foo"], dep_bundles: vec!["foo_dep"] },
                LazyPlugin { id : "bar", plugin : "bar plugin", startup: "bar startup", config: "bar config", deps: vec!["foo"], dep_bundles: vec!["bar_dep"] },
            ],
        ),
        case(vec![],vec![]),
    )]
    fn derive_lazy(arg: Vec<LazyPlugin>, mut exp: Vec<LazyPlugin>) {
        // arrange:
        exp.sort();

        // act:
        let mut act = derive(arg).unwrap();
        act.sort();

        // assert:
        assert_eq!(act, exp);
    }

    #[rstest(arg, exp,
        case(
            vec![
                Bundle { id : "foo", plugins : vec![], startup: "", config: "", deps: vec![], dep_bundles: vec![] },
                Bundle { id : "bar", plugins : vec![], startup: "", config: "", deps: vec![], dep_bundles: vec![] },
                Bundle { id : "foo", plugins : vec!["foo_plugins"], startup: "foo startup", config: "foo config", deps: vec!["foo"], dep_bundles: vec!["foo_dep"] },
                Bundle { id : "bar", plugins : vec!["bar_plugins"], startup: "bar startup", config: "bar config", deps: vec!["foo"], dep_bundles: vec!["bar_dep"] },
            ],
            vec![
                Bundle { id : "foo", plugins : vec!["foo_plugins"], startup: "foo startup", config: "foo config", deps: vec!["foo"], dep_bundles: vec!["foo_dep"] },
                Bundle { id : "bar", plugins : vec!["bar_plugins"], startup: "bar startup", config: "bar config", deps: vec!["foo"], dep_bundles: vec!["bar_dep"] },
            ],
        ),
        case(vec![],vec![]),
    )]
    fn derive_bundle(arg: Vec<Bundle>, mut exp: Vec<Bundle>) {
        // arrange:
        exp.sort();

        // act:
        let mut act = derive(arg).unwrap();
        act.sort();

        // assert:
        assert_eq!(act, exp);
    }

    #[rstest(arg,
        case(
             OboroConfig {
                 startup_plugins: vec![StartupPlugin {id: "foo", ..Default::default()},],
                 lazy_plugins: vec![LazyPlugin {id: "bar", ..Default::default()},],
                 bundles: vec![Bundle {id: "baz", ..Default::default()},] ,..Default::default()
            }
        ),
    #[should_panic]
        case(
            OboroConfig {
                startup_plugins: vec![StartupPlugin {id: "foo", ..Default::default()},],
                lazy_plugins: vec![LazyPlugin {id: "foo", ..Default::default()},], ..Default::default()
            }
        ),
    #[should_panic]
        case(
            OboroConfig {
                startup_plugins: vec![StartupPlugin {id: "foo", ..Default::default()},],
                bundles: vec![ Bundle { id: "foo", ..Default::default()},], ..Default::default()
            }
        ),
    #[should_panic]
        case(
            OboroConfig {
                lazy_plugins: vec![LazyPlugin {id: "foo", ..Default::default()},],
                bundles: vec![Bundle {id: "foo", ..Default::default()},], ..Default::default()
            }
        ),
     )]
    fn validate_config(arg: OboroConfig) {
        validate(&arg).unwrap();
    }

    #[test]
    fn map_config() {
        // arrange:
        let src = input::OboroPluginConfig {
            start_plugins: vec![input::StartPlugin {
                id: String::from("foo"),
                plugin: String::from("foo_plugin"),
                startup: String::from("foo startup"),
            }],
            opt_plugins: vec![
                input::OptPlugin {
                    id: String::from("bar"),
                    plugin: String::from("bar_plugin"),
                    startup: String::from("bar startup"),
                    config: String::from("bar config"),
                    deps: vec![String::from("baz")],
                    dep_bundles: vec![String::from("hoge")],
                    cmds: vec![String::from("bar_cmd")],
                    mods: vec![String::from("bar_mod")],
                    evs: vec![String::from("bar_ev")],
                    fts: vec![String::from("bar_ft")],
                    lazy: true,
                },
                input::OptPlugin {
                    id: String::from("bar"),
                    ..Default::default()
                },
                input::OptPlugin {
                    id: String::from("baz"),
                    ..Default::default()
                },
                input::OptPlugin {
                    id: String::from("qux"),
                    ..Default::default()
                },
                input::OptPlugin {
                    id: String::from("quux"),
                    ..Default::default()
                },
            ],
            bundles: vec![
                input::Bundle {
                    id: String::from("hoge"),
                    plugins: vec![String::from("bar"), String::from("qux")],
                    startup: String::from("hoge startup"),
                    config: String::from("hoge config"),
                    deps: vec![String::from("quux")],
                    dep_bundles: vec![String::from("huga")],
                    cmds: vec![String::from("hoge_cmd")],
                    mods: vec![String::from("hoge_mod")],
                    evs: vec![String::from("hoge_ev")],
                    fts: vec![String::from("hoge_ft")],
                    lazy: true,
                },
                input::Bundle {
                    id: String::from("hoge"),
                    ..Default::default()
                },
                input::Bundle {
                    id: String::from("huga"),
                    ..Default::default()
                },
            ],
        };
        let exp = OboroConfig {
            startup_plugins: vec![StartupPlugin {
                id: "foo",
                plugin: "foo_plugin",
                startup: "foo startup",
            }],
            lazy_plugins: vec![
                LazyPlugin {
                    id: "bar",
                    plugin: "bar_plugin",
                    startup: "bar startup",
                    config: "bar config",
                    deps: vec!["baz"],
                    dep_bundles: vec!["hoge"],
                },
                LazyPlugin {
                    id: "bar",
                    ..Default::default()
                },
                LazyPlugin {
                    id: "baz",
                    ..Default::default()
                },
                LazyPlugin {
                    id: "qux",
                    ..Default::default()
                },
                LazyPlugin {
                    id: "quux",
                    ..Default::default()
                },
            ],
            bundles: vec![
                Bundle {
                    id: "hoge",
                    plugins: vec!["bar", "qux"],
                    startup: "hoge startup",
                    config: "hoge config",
                    deps: vec!["quux"],
                    dep_bundles: vec!["huga"],
                },
                Bundle {
                    id: "hoge",
                    ..Default::default()
                },
                Bundle {
                    id: "huga",
                    ..Default::default()
                },
            ],
            cmds: vec![],
            mods: vec![],
            evs: vec![],
            fts: vec![],
            cmd_map: HashMap::from([("bar_cmd", vec!["bar"]), ("hoge_cmd", vec!["hoge"])]),
            mod_map: HashMap::from([("bar_mod", vec!["bar"]), ("hoge_mod", vec!["hoge"])]),
            ev_map: HashMap::from([("bar_ev", vec!["bar"]), ("hoge_ev", vec!["hoge"])]),
            ft_map: HashMap::from([("bar_ft", vec!["bar"]), ("hoge_ft", vec!["hoge"])]),
            lazys: vec!["bar", "hoge"],
        };

        // act:
        let act = map(&src);

        // assert:
        assert_eq!(act, exp);
    }
}
