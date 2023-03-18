mod config;
mod generator;

use crate::config::input::OboroPluginConfig;
use crate::config::resolve;
use crate::generator::generate;
use std::{env, fs};

fn main() {
    println!("start");
    let args: Vec<String> = env::args().collect();
    let input_json_path = &args[1];
    let output_dir = &args[2];
    println!(
        "input json: {}, output dir: {}",
        input_json_path, output_dir
    );

    let input_json_text = fs::read_to_string(input_json_path).unwrap();
    let config_src = serde_json::from_str::<OboroPluginConfig>(&input_json_text).unwrap();
    let config = resolve(&config_src).unwrap();

    generate(&config, output_dir).unwrap();
    println!("completed!")
}
