
extern crate qamd;
extern crate toml;
extern crate serde;
extern crate serde_json;

use qamd::read;
use qamd::config::{ Config, Valid };
// use qamd::config::{ VariableConfig, ValueConfig, Setting, Level };
// use qamd::report::Report;

use std::env;
use std::process;

// use std::io;
// use std::error;
use std::io::prelude::*;
use std::fs::File;

macro_rules! ok(($expression:expr) => ($expression.unwrap()));

fn main() {
    if env::args().count() < 2 {
        println!("usage:");
        println!("\tqamd path/to/data/file.ext [path/to/config/file.toml]");
        process::exit(1);
    }

    let (file_path, config_path) = match env::args().count() {
        2 => (ok!(env::args().nth(1)),
              "config.toml".into()),
        3 => (ok!(env::args().nth(1)),
              ok!(env::args().nth(2))),
        _ => ("".into(),
              "".into())
    };

    // println!("{}, {}", file_path, config_path);
    /*
    let config = Config {
        variable_config: VariableConfig {
            odd_characters: Setting::<Option<Vec<String>>> {
                setting: Some(vec!("!", "#", "  ", "@", "ë", "ç", "ô", "ü")
                              .iter()
                              .map(|x| x.to_string())
                              .collect::<Vec<String>>()),
                level: Level::Warn
            },
            missing_variable_labels: Setting::<bool> {
                setting: true,
                level: Level::Warn
            },
        },
        value_config: ValueConfig {
            odd_characters: Setting::<Option<Vec<String>>> {
                setting: Some(vec!("!", "#", "  ", "@", "ë", "ç", "ô", "ü")
                              .iter()
                              .map(|x| x.to_string())
                              .collect::<Vec<String>>()),
                level: Level::Warn
            },
            system_missing_value_threshold: Setting::<Option<i32>> {
                setting: Some(25),
                level: Level::Fail
            },
        },
    };

    println!("{}", toml::to_string(&config).unwrap());
    */

    match parse_config(&config_path) {
        Ok(config) => {
            //println!("Config: {:#?}", config);

            let report = ok!(read(&file_path, &config));
            let serialised = ok!(serde_json::to_string(&report));

            println!("{}", serialised);
        },
        Err(err) => println!("{:?}", err),
    }
}

fn parse_config(path: &str) -> Result<Config, String> {
    let mut f = File::open(path)
        .expect(&format!("Failed to open file for reading: {}", path));

    let mut buffer: String = "".into();
    f.read_to_string(&mut buffer)
        .expect(&format!("Failed to read data from file: {}", path));

    match toml::from_str::<Config>(&buffer) {
        Ok(config) => {
            let valid = config.validate();
            match valid {
                Ok(()) => Ok(config),
                Err(err) => Err(format!("Invalid config: {}", err)),
            }
        },
        Err(err)   => Err(format!("Failed to parse toml: {}", err)),
    }
}

