extern crate bindgen;
extern crate pkg_config;

use std::path::PathBuf;
use std::env;
use std::process::Command;

const LIBS: [&'static str; 3]= [
    "static=ReadStat",
    "dylib=iconv",
    "dylib=z",
];

const LIB_SEARCH_PATHS: [&'static str; 1] = [
    "/usr/lib",
];

const READSTAT_URL: &'static str = "https://github.com/WizardMac/ReadStat.git";
const READSTAT_DIR: &'static str = "ReadStat/";

macro_rules! get(($name:expr) => (env::var($name).unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("qamd/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("qamd/build.rs:{}: ", $fmt),
                                          line!(), $($arg)*));
}
macro_rules! log_var(($var:ident) =>
                     (log!(concat!(stringify!($var), " = {:?}"), $var)));

/// Setup the rustc link search directories & library
fn main() {
    LIBS.iter()
        .for_each(|lib| println!("cargo:rustc-link-lib={}", lib));

    LIB_SEARCH_PATHS.iter()
        .for_each(|lib| println!("cargo:rustc-link-search={}", lib));

    let out_path = PathBuf::from(&get!("OUT_DIR"));
    log_var!(out_path);
    let mut readstat_search_path = out_path.join(&READSTAT_DIR);
    readstat_search_path.push("src");
    println!("cargo:rustc-link-search={}", &readstat_search_path.display());

    get_readstat();

    run("make", |command| {
        command.current_dir(&out_path)
    });

    generate_bindings();
}

/// Clone the readstat directory if it isn't already present.
fn get_readstat() {
    let out_path = PathBuf::from(&get!("OUT_DIR"));
    let clone_dir = out_path.join(&READSTAT_DIR);

    // Clone repo
    if !&clone_dir.exists() {
        run("git", |command| {
            command.current_dir(&out_path)
                .arg("clone")
                .arg(&READSTAT_URL)
        });

        run("cp", |command| {
            command.arg("Makefile")
                .arg(&out_path)
        });
    }
}

/// Use bindgen to generate the rust code required to interact with the C code.
fn generate_bindings() {
    let out_path = PathBuf::from(&get!("OUT_DIR"));
    let bindings_file = out_path.join("bindings.rs");

    if !&bindings_file.exists() {
        log!("Attempting to generate bindings via bindgen.");
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .whitelisted_function(r"readstat_[a-z0-9_]+")
            .whitelisted_type(r"readstat_[a-z]+_t")
            .whitelisted_var(r"READSTAT_HANDLER_[A-Z]+")
            .generate()
            .expect("Unable to generate bindings");

        log!("Bindings generated bindings.");
        log!("Attempting to write to file {:?}", &bindings_file);

        bindings.write_to_file(&bindings_file)
            .expect("Couldn't write bindings!");

        log!("Successfully written bindings to {:?}", &bindings_file);
    } else {
        log!("Bindings already generated. Skipping.");
    }
}

/// Build and run a Command and log the result.
fn run<F>(name: &str, mut configure: F) where F: FnMut(&mut Command) -> &mut Command {
    let mut command = Command::new(name);
    let configured = configure(&mut command);

    log!("Executing {:?}", configured);
    if !configured.status().unwrap().success() {
        panic!("Failed to execute {:?}", configured);
    }
    log!("Command {:?} finished sucessfully.", configured);
}

