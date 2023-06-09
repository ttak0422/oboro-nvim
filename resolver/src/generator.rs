use crate::config::output::OboroConfig;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::{create_dir, File};
use std::io::Write;

/// vector to table.
fn to_lua_table(v: &[&str]) -> String {
    v.iter()
        .fold(String::from("{"), |acc, x| acc + "'" + x + "',")
        + "}"
}

/// generate startup config.
fn gen_startup(config: &OboroConfig, root: &str) -> Result<()> {
    let path = String::from(root) + "/startup";
    let mut file = File::create(&path)?;

    // startup
    for plugin in config.startup_plugins.iter() {
        if plugin.startup.is_empty() {
            continue;
        }
        write!(file, "-- {}\n{}\n", plugin.id, plugin.startup)?;
    }

    // lazy
    for plugin in config.lazy_plugins.iter() {
        if plugin.startup.is_empty() {
            continue;
        }
        write!(file, "-- {}\n{}\n", plugin.id, plugin.startup)?;
    }

    // bundle
    for bundle in config.bundles.iter() {
        if bundle.startup.is_empty() {
            continue;
        }
        write!(file, "-- {}\n{}\n", bundle.id, bundle.startup)?;
    }

    println!("write: {}", &path);
    file.flush().map_err(|err| anyhow!(err))
}

/// generate configs.
fn gen_config(config: &OboroConfig, root: &str) -> Result<()> {
    // setup
    create_dir(String::from(root) + "/pre_cfgs")?;
    create_dir(String::from(root) + "/cfgs")?;
    create_dir(String::from(root) + "/deps")?;
    create_dir(String::from(root) + "/plugin")?;
    create_dir(String::from(root) + "/plugins")?;

    // lazy
    for plugin in config.lazy_plugins.iter() {
        let pre_cfg_path = String::from(root) + "/pre_cfgs/" + plugin.id;
        let cfg_path = String::from(root) + "/cfgs/" + plugin.id;
        let deps_path = String::from(root) + "/deps/" + plugin.id;
        let plugin_path = String::from(root) + "/plugin/" + plugin.id;
        let plugins_path = String::from(root) + "/plugins/" + plugin.id;
        let mut pre_cfg_file = File::create(&pre_cfg_path)?;
        let mut cfg_file = File::create(&cfg_path)?;
        let mut deps_file = File::create(&deps_path)?;
        let mut plugin_file = File::create(&plugin_path)?;
        let mut plugins_file = File::create(&plugins_path)?;
        write!(pre_cfg_file, "{}", plugin.pre_config)?;
        write!(cfg_file, "{}", plugin.config)?;
        write!(
            deps_file,
            "return {}",
            to_lua_table(&([&plugin.deps[..], &plugin.dep_bundles[..]]).concat())
        )?;
        write!(plugin_file, "return '{}'", plugin.id)?;
        write!(plugins_file, "return {{}}")?;
        println!("write: {}", &pre_cfg_path);
        println!("write: {}", &cfg_path);
        println!("write: {}", &deps_path);
        println!("write: {}", &plugin_path);
        println!("write: {}", &plugins_path);
        pre_cfg_file.flush().map_err(|err| anyhow!(err))?;
        cfg_file.flush().map_err(|err| anyhow!(err))?;
        deps_file.flush().map_err(|err| anyhow!(err))?;
        plugin_file.flush().map_err(|err| anyhow!(err))?;
        plugins_file.flush().map_err(|err| anyhow!(err))?;
    }

    // bundle
    for bundle in config.bundles.iter() {
        let pre_cfg_path = String::from(root) + "/pre_cfgs/" + bundle.id;
        let cfg_path = String::from(root) + "/cfgs/" + bundle.id;
        let deps_path = String::from(root) + "/deps/" + bundle.id;
        let plugin_path = String::from(root) + "/plugin/" + bundle.id;
        let plugins_path = String::from(root) + "/plugins/" + bundle.id;
        let mut pre_cfg_file = File::create(&pre_cfg_path)?;
        let mut cfg_file = File::create(&cfg_path)?;
        let mut deps_file = File::create(&deps_path)?;
        let mut plugin_file = File::create(&plugin_path)?;
        let mut plugins_file = File::create(&plugins_path)?;
        write!(pre_cfg_file, "{}", bundle.pre_config)?;
        write!(cfg_file, "{}", bundle.config)?;
        // write!(deps_file, "return {}", to_lua_table(bundle.deps))?;
        write!(
            deps_file,
            "return {}",
            to_lua_table(&([&bundle.deps[..], &bundle.dep_bundles[..]]).concat())
        )?;
        write!(plugin_file, "return nil")?;
        write!(plugins_file, "return {}", to_lua_table(&bundle.plugins))?;
        println!("write: {}", &pre_cfg_path);
        println!("write: {}", &cfg_path);
        println!("write: {}", &deps_path);
        println!("write: {}", &plugin_path);
        println!("write: {}", &plugins_path);
        pre_cfg_file.flush().map_err(|err| anyhow!(err))?;
        cfg_file.flush().map_err(|err| anyhow!(err))?;
        deps_file.flush().map_err(|err| anyhow!(err))?;
        plugin_file.flush().map_err(|err| anyhow!(err))?;
        plugins_file.flush().map_err(|err| anyhow!(err))?;
    }
    Ok(())
}

/// generate key value (vector) pair.
fn gen_kvp(kvp: &HashMap<&str, Vec<&str>>, path_prefix: &str) -> Result<()> {
    for (k, v) in kvp {
        let path = String::from(path_prefix) + k;
        let mut file = File::create(&path)?;
        write!(file, "return {}", to_lua_table(v))?;
        println!("write: {}", &path);
        file.flush().map_err(|err| anyhow!(err))?;
    }
    Ok(())
}

/// generate configs.
pub fn generate(config: &OboroConfig, root: &str) -> Result<()> {
    // start
    gen_startup(config, root)?;

    // opt
    gen_config(config, root)?;

    // modules
    let mod_tbl_path = String::from(root) + "/mod_tbl";
    let mut mod_tbl_file = File::create(&mod_tbl_path)?;
    write!(mod_tbl_file, "return {}", to_lua_table(&config.mods))?;
    println!("write: {}", &mod_tbl_path);
    mod_tbl_file.flush().map_err(|err| anyhow!(err))?;
    create_dir(String::from(root) + "/mods")?;
    let mods_path = String::from(root) + "/mods/";
    gen_kvp(&config.mod_map, &mods_path)?;

    // events
    let ev_tbl_path = String::from(root) + "/ev_tbl";
    let mut evs_file = File::create(&ev_tbl_path)?;
    write!(evs_file, "return {}", to_lua_table(&config.evs))?;
    println!("write: {}", &ev_tbl_path);
    evs_file.flush().map_err(|err| anyhow!(err))?;
    create_dir(String::from(root) + "/evs")?;
    let evs_path = String::from(root) + "/evs/";
    gen_kvp(&config.ev_map, &evs_path)?;

    // filetypes
    let ft_tbl_path = String::from(root) + "/ft_tbl";
    let mut fts_file = File::create(&ft_tbl_path)?;
    write!(fts_file, "return {}", to_lua_table(&config.fts))?;
    println!("write: {}", &ft_tbl_path);
    fts_file.flush().map_err(|err| anyhow!(err))?;
    create_dir(String::from(root) + "/fts")?;
    let fts_path = String::from(root) + "/fts/";
    gen_kvp(&config.ft_map, &fts_path)?;

    // commands
    let cmd_tbl_path = String::from(root) + "/cmd_tbl";
    let mut cmds_file = File::create(&cmd_tbl_path)?;
    write!(cmds_file, "return {}", to_lua_table(&config.cmds))?;
    println!("write: {}", &cmd_tbl_path);
    cmds_file.flush().map_err(|err| anyhow!(err))?;
    create_dir(String::from(root) + "/cmds")?;
    let cmds_path = String::from(root) + "/cmds/";
    gen_kvp(&config.cmd_map, &cmds_path)?;

    // lazy
    let lazy_path = String::from(root) + "/lazy";
    let mut lazy_file = File::create(&lazy_path)?;
    write!(lazy_file, "return {}", to_lua_table(&config.lazys))?;
    println!("write: {}", lazy_path);
    lazy_file.flush().map_err(|err| anyhow!(err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(arg, exp,
        case(vec!["foo"], "{'foo',}"),
        case(vec!["foo", "bar"], "{'foo','bar',}"),
        case(vec![], "{}"),
    )]
    fn vector_to_table(arg: Vec<&str>, exp: String) {
        // act:
        let act = to_lua_table(&arg);

        // assert:
        assert_eq!(act, exp);
    }
}
