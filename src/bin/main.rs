
extern crate qamd;
extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate toml;

use qamd::readstat::read::read;
use qamd::config::{ Config, Valid };
use qamd::html::to_html;
// use qamd::config::{ VariableConfig, ValueConfig, Setting, Level };
// use qamd::report::Report;

use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use clap::{ Arg, App };

macro_rules! ok(($expression:expr) =>
                ($expression.unwrap()));

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
                        .arg(Arg::with_name("output")
                             .short("o")
                             .long("output")
                             .value_name("FILE")
                             .help("Sets an optional output file.")
                             .takes_value(true))
                        .arg(Arg::with_name("output-format")
                             .long("output-format")
                             .value_name("FILE_TYPE")
                             .help("Sets the output format. Can be either json or html Default to JSON.")
                             .takes_value(true)
                             .possible_values(&["json", "html"]))
                        .arg(Arg::with_name("locators")
                             .short("l")
                             .long("include-locators")
                             .help(format!("{} {} {}",
                                           "If set the summary report includes",
                                           "the index of the value(s) & or",
                                           "variable(s) for any failed checks.").as_str()))
                        .arg(Arg::with_name("disable-progress")
                             .short("p")
                             .long("disable-progress")
                             .help(format!("{} {}",
                                           "If set, disables the progress bar.",
                                           "Useful if running inside scripts").as_str()))
                        .get_matches();

    let file_path = matches
        .value_of("input")
        .unwrap();
    let config_path = matches
        .value_of("config")
        .unwrap_or("config.toml");
    let output_path = matches
        .value_of("output");
    let output_format = matches
        .value_of("output-format")
        .unwrap_or("json");

    let include_locators = match matches.occurrences_of("locators") {
        0 => false,
        _ => true,
    };

    let progress = match matches.occurrences_of("disable-progress") {
        0 => true,
        _ => false,
    };

    // println!("file_path: {}\nconfig_path: {}\nlocators: {}", file_path, config_path, include_locators);

    match parse_config(&config_path) {
        Ok(ref mut config) => {
            //println!("Config: {:#?}", config);

            config.include_locators = override_config(config.include_locators,
                                                  include_locators);
            config.progress = override_config(config.progress,
                                          progress);

            let report = ok!(read(&file_path, &config));

            let serialised = match output_format {
                "json" => ok!(serde_json::to_string(&report)),
                "html" => to_html(&report),
                _ => "".to_string(),
            };

            let _ = match output_path {
                Some(path) => write_to_file(&path, &serialised),
                None => {
                    println!("{}", serialised);
                    Ok(())
                },
            };
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

fn override_config<T>(option: Option<T>, value: T) -> Option<T> {
    if option.is_none() {
        Some(value)
    } else {
        option
    }
}

fn write_to_file(path: &str, contents: &str) -> Result<(), io::Error> {
    let f = File::create(path)?;

    {
        let mut writer = BufWriter::new(f);

        let _ = writer.write(contents.as_bytes());
    }

    Ok(())
}

