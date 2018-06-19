
extern crate qamd;
extern crate clap;
extern crate toml;
extern crate serde;
extern crate serde_json;

use qamd::read;
use qamd::config::{ Config, Valid };
// use qamd::config::{ VariableConfig, ValueConfig, Setting, Level };
// use qamd::report::Report;

use std::io::prelude::*;
use std::fs::File;

use clap::{ Arg, App };

macro_rules! ok(($expression:expr) => ($expression.unwrap()));



fn main() {
    let matches = App::new("QA My Data")
                        .version("0.1.0")
                        .author("Myles Offord")
                        .about(format!("{} {} {}",
                                       "Produces a summary report of common",
                                       "issues to provide a higher standard of statistical data.",
                                       "Currently supports SPSS, STATA, SAS and (soon™) CSV!").as_str())
                        .arg(Arg::with_name("input")
                             .help("Sets the input file to use.")
                             .required(true)
                             .index(1))
                        .arg(Arg::with_name("config")
                             .short("c")
                             .long("config")
                             .value_name("FILE")
                             .help("Sets a custom config file")
                             .takes_value(true))
                        .arg(Arg::with_name("locators")
                             .short("l")
                             .long("include-locators")
                             .help(format!("{} {} {}",
                                           "If set the summary report includes",
                                           "the index of the value(s) & or",
                                           "variable(s) for any failed checks.").as_str()))
                        .get_matches();

    let file_path = matches
        .value_of("input")
        .unwrap();
    let config_path = matches
        .value_of("config")
        .unwrap_or("config.toml");

    let include_locators = match matches.occurrences_of("locators") {
        0 => false,
        1 => true,
        _ => true,
    };

    // println!("file_path: {}\nconfig_path: {}\nlocators: {}", file_path, config_path, include_locators);

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

