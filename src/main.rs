
extern crate qamd;
extern crate toml;
extern crate serde;
extern crate serde_json;

use qamd::read_sav;
use qamd::config::Config;
//use qamd::report::Report;

use std::env;
use std::process;

//use std::io;
//use std::error;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    if env::args().count() < 2 {
        println!("usage:");
        println!("\tqamd path/to/data/file.ext [path/to/config/file.toml]");
        process::exit(1);
    }

    let (file_path, config_path) = match env::args().count() {
        2 => (env::args().nth(1).unwrap(),
              "config.toml".into()),
        3 => (env::args().nth(1).unwrap(),
              env::args().nth(2).unwrap()),
        _ => ("".into(),
              "".into())
    };

    // println!("{}, {}", file_path, config_path);

    let config: Config = parse_config(&config_path).unwrap();
    // println!("Config: {:?}", config);

    let report = read_sav(&file_path, &config).unwrap();
    let serialised = serde_json::to_string(&report).unwrap();

    println!("{}", serialised);
}

fn parse_config(path: &str) -> Result<Config, String> {
    let mut f = File::open(path)
        .expect(&format!("Failed to open file for reading: {}", path));

    let mut buffer: String = "".into();
    f.read_to_string(&mut buffer)
        .expect(&format!("Failed to read data from file: {}", path));

    Ok(toml::from_str(&buffer)
        .expect("failed to parse toml"))
}

