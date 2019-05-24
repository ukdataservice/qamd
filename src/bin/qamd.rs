extern crate clap;
extern crate qamd;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate toml;

use qamd::config::{Config, Valid};
use qamd::html::to_html;
use qamd::readstat::read::read;

use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

static DEFAULT_CONFIG: &'static str = include_str!("../../config.toml");
static ABOUT_TEXT: &'static str = concat!(
    "QAMyData offers a free easy-to-use",
    " tool that automatically detects some",
    " of the most common problems in",
    " survey and other numeric data and",
    " creates a ‘data health check’,",
    " assisting with the clean up of data",
    " and providing an assurance that data",
    " is of a high quality."
);
static RUN_ABOUT_TEXT: &'static str = concat!(
    "Run QAMyData on a target file.",
    " To show usage use, qamd help run. "
);
static INIT_ABOUT_TEXT: &'static str = concat!(
    "Scaffold a new QAMyData project with including",
    " the default config file.\n\n",
    "This command will create the following directory tree:",
    "\n\t.",
    "\n\t├── config",
    "\n\t│   └── default.toml",
    "\n\t├── data",
    "\n\t│   └── test_data",
    "\n\t└── dictionaries",
    "\n\t    └── basic_english.txt",
);

fn main() {
    let matches = parse_arguments();

    match matches.subcommand() {
        ("init", Some(_)) => init(),
        ("run", Some(run_matches)) => run(run_matches),
        _ => {}
    }
}

fn parse_arguments() -> clap::ArgMatches<'static> {
    App::new("QA My Data")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Myles Offord - moffor@essex.ac.uk")
        .about(ABOUT_TEXT)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("init").about(INIT_ABOUT_TEXT))
        .subcommand(
            SubCommand::with_name("run")
                .about(RUN_ABOUT_TEXT)
                .arg(
                    Arg::with_name("input")
                        .help("Sets the input file to use.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("Sets a custom config file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("FILE")
                        .help("Sets an optional output file.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("output-format")
                        .long("output-format")
                        .value_name("FILE_TYPE")
                        .help("Sets the output format. Can be either JSON or HTML. If ommited, defaults to HTML.")
                        .takes_value(true)
                        .possible_values(&["json", "html"]),
                )
                .arg(
                    Arg::with_name("metadata-only")
                        .short("m")
                        .long("metadata-only")
                        .help(
                            concat!(
                                "If set the output will only inlcude metadata",
                                " from the file and the number of passes and",
                                " failures for each check. Data for locating",
                                " each failure will be ommited."
                            )
                        ),
                )
                .arg(
                    Arg::with_name("disable-progress")
                        .short("p")
                        .long("disable-progress")
                        .help(
                            format!(
                                "{} {}",
                                "If set, disables the progress bar.",
                                "Useful if running inside scripts"
                            )
                            .as_str(),
                        ),
                ),
        )
        .get_matches()
}

fn init() {
    let base_path = std::env::current_dir().expect(concat!(
        "Insufficent permissions to access the current",
        " directory or directory doesn't exist."
    ));

    let dirs: [&'static str; 4] = ["config", "dictionaries", "data", "data/test"];

    for dir in dirs.iter() {
        match fs::create_dir(base_path.join(dir)) {
            Ok(_) => (),
            Err(_) => {
                println!("Directory {} already exists, exiting...", dir);
                std::process::exit(1);
            }
        }
    }

    fs::write(
        base_path.join("config").join("default.toml"),
        DEFAULT_CONFIG,
    )
    .expect("Failed to write config/default.toml");

    let github = "https://github.com/ukdataservice/qamd/blob/master/".to_string();
    let test_data_dir = base_path.join("data").join("test");

    let words = "https://raw.githubusercontent.com/dwyl/english-words/master/words.txt";
    let mtcars_stata = format!("{}{}", &github, "test/mtcars.dta?raw=true");
    let mtcars_spss = format!("{}{}", &github, "test/mtcars.sav?raw=true");
    let mtcars_sas = format!("{}{}", &github, "test/mtcars.sas?raw=true");
    let mtcars_csv = format!("{}{}", &github, "test/mtcars.csv?raw=true");

    get_file(words, base_path.join("dictionaries").join("en.txt"));
    get_file(&mtcars_stata, test_data_dir.join("mtcars.dta"));
    get_file(&mtcars_spss, test_data_dir.join("mtcars.sav"));
    get_file(&mtcars_sas, test_data_dir.join("mtcars.sas7bdat"));
    get_file(&mtcars_csv, test_data_dir.join("mtcars.csv"));
}

fn get_file(uri: &str, file: std::path::PathBuf) {
    match reqwest::get(uri) {
        Ok(mut res) => {
            let mut buf: Vec<u8> = vec![];
            res.copy_to(&mut buf)
                .expect("Failed to write response body to buffer.");

            fs::write(&file, buf).expect(&format!("Failed to write {}", &file.to_str().unwrap()));
        }
        Err(_) => println!(
            concat!("Warning: Couldn't get {}", " You can find it here: {}"),
            &file.to_str().unwrap(),
            uri
        ),
    }
}

fn run(matches: &ArgMatches) {
    let config_file = match matches.value_of("config") {
        Some(config_path) => {
            read_file(config_path).expect(&format!("Failed to read file {}", config_path))
        }
        None => String::from(DEFAULT_CONFIG),
    };

    let file_path = matches.value_of("input").unwrap();
    let output_path = matches.value_of("output");
    let output_format = matches.value_of("output-format").unwrap_or("html");

    let metadata_only = match matches.occurrences_of("metadata-only") {
        0 => false,
        _ => true,
    };

    let progress = match matches.occurrences_of("disable-progress") {
        0 => true,
        _ => false,
    };

    match parse_config(&config_file) {
        Ok(ref mut config) => {
            config.metadata_only = override_config(config.metadata_only, metadata_only);
            config.progress = override_config(config.progress, progress);

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
                        }
                    };
                }
                Err(err) => eprintln!("{} : {}:{}:{}", err, file!(), line!(), column!()),
            };
        }
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
        }
        Err(err) => Err(format!("Failed to parse config: {}", err)),
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

    let mut writer = BufWriter::new(f);
    let _ = writer.write(contents.as_bytes());

    Ok(())
}
