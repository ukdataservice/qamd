
extern crate qamd;
extern crate toml;

use qamd::read_sav;
use qamd::config::Config;
use qamd::report::Report;

use std::env;
use std::process;

//use std::io;
use std::error;
use std::io::prelude::*;
use std::fs::File;

fn main() {
    if env::args().count() < 2 {
        println!("usage:\n\tqamd path/to/data/file.ext [path/to/config/file.toml]");
        process::exit(1);
    }

    let (file_path, config_path) = match env::args().count() {
        2 => (env::args().nth(1).unwrap(),
              "".into()),
        3 => (env::args().nth(1).unwrap(),
              env::args().nth(2).unwrap()),
        _ => ("".into(),
              "".into())
    };

    println!("{}, {}", file_path, config_path);

    let config: Config = parse_config(&config_path).unwrap();
    println!("Config: {:?}", config);

    let report = read_sav(&file_path).unwrap();
    println!("raw_case_count: {}", report.metadata.raw_case_count);
}

fn parse_config<>(path: &str) -> Result<Config, toml::de::Error> {
    let mut f = File::open(path).unwrap();
    let mut buffer: String = "".into();
    f.read_to_string(&mut buffer).unwrap();

    toml::from_str(&buffer)
}

