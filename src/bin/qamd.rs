
extern crate qamd;
extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate toml;

use qamd::readstat::read::read;
use qamd::config::{ Config, Valid };
use qamd::html::to_html;

use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use clap::{ Arg, App };

static DEFAULT_CONFIG: &'static str = include_str!("../../config.toml");
static ABOUT_TEXT: &'static str =
    concat!("QAMyData offers a free easy-to-use",
            " tool that automatically detects some",
            " of the most common problems in",
            " survey and other numeric data and",
            " creates a ‘data health check’,",
            " assisting with the clean up of data",
            " and providing an assurance that data",
            " is of a high quality.");

fn main() {
    let matches = App::new("QA My Data")
                        .version("0.1.0")
                        .author("Myles Offord - moffor@essex.ac.uk")
                        .about(ABOUT_TEXT)
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

    let config_file = match matches.value_of("config") {
        Some(config_path) => read_file(config_path)
            .expect(&format!("Failed to read file {}", config_path)),
        None => String::from(DEFAULT_CONFIG),
    };

    let file_path = matches
        .value_of("input")
        .unwrap();
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

    match parse_config(&config_file) {
        Ok(ref mut config) => {
            config.include_locators = override_config(config.include_locators,
                                                      include_locators);
            config.progress = override_config(config.progress,
                                              progress);

            match read(&file_path, &config) {
                Ok(report) => {
                    let serialised = match output_format {
                        "json" => serde_json::to_string(&report).unwrap(),
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
                Err(err) => eprintln!("{} : {}:{}:{}", err, file!(), line!(), column!()),
            };

        },
        Err(err) => eprintln!("{} : {}", err, line!()),
    }
}

fn read_file(path: &str) -> io::Result<String> {
    let mut f = File::open(path)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    Ok(buffer)
}

fn parse_config(config_file: &str) -> Result<Config, String> {
    match toml::from_str::<Config>(&config_file) {
        Ok(config) => {
            let valid = config.validate();
            match valid {
                Ok(()) => Ok(config),
                Err(err) => Err(format!("Invalid config: {}", err)),
            }
        },
        Err(err) => Err(format!("Failed to parse toml: {}", err)),
    }
}

fn override_config<T>(option: Option<T>, value: T) -> Option<T> {
    if option.is_none() {
        Some(value)
    } else {
        option
    }
}

fn write_to_file(path: &str, contents: &str) -> io::Result<()> {
    let f = File::create(path)?;

    {
        let mut writer = BufWriter::new(f);

        let _ = writer.write(contents.as_bytes());
    }

    Ok(())
}

