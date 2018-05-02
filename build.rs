#![allow(dead_code)]

extern crate bindgen;
extern crate pkg_config;

use std::fs::File;
use std::path::{ Path, PathBuf };
use std::process::Command;
use std::{ env, fs };

const LIBRARY: &'static str = "readstat";
const FRAMEWORK_LIBRARY: &'static str = "readstat_framework";
const TARGET: &'static str = "readstat:libreadstat.so";
// const VERSION: &'static str = "0.1.1";
// TAG may differ from VERSION, therefore sperate.
// const TAG: &'static str = "v0.1.1"; 
const REPOSITORY: &'static str = "https://github.com/WizardMac/ReadStat.git";

macro_rules! get(($name:expr) => (ok!(env::var($name))));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("libreadstat/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("libreadstat/build.rs:{}: ", $fmt),
                                          line!(), $($arg)*));
}
macro_rules! log_var(($var:ident) =>
                     (log!(concat!(stringify!($var), " = {:?}"), $var)));

fn main() {
    if pkg_config::find_library(LIBRARY).is_ok() {
        log!("Returning early because {} was already found", LIBRARY);
        return;
    }

    // decided to elide build from src as it should really be down to the
    // user to install the desired version. Currently using master 8bab9b2
    // build_from_src();

    let lib_dir = PathBuf::from("/usr/local/lib");
    log_var!(lib_dir);
    let framework_library_path =
        lib_dir.join(format!("lib{}.so", FRAMEWORK_LIBRARY));
    log_var!(framework_library_path);

    if !framework_library_path.exists() {
        log!("ReadStat not installed, exiting.");
    }

    println!("cargo:rustc-link-lib=dylib={}", LIBRARY);
    println!("cargo:rustc-link-search=/usr/local/lib");

    generate_bindings();
}

/// build readstat from source and `` it.
fn build_from_src() {
    let output = PathBuf::from(&get!("OUT_DIR"));
    log_var!(output);
    let source = PathBuf::from(&get!("CARGO_MANIFEST_DIR"))
        .join("target/source");
    log_var!(source);
    let lib_dir = PathBuf::from("/usr/local/lib");
    log_var!(lib_dir);

    if lib_dir.exists() {
        log!("Directory {:?} already exists", lib_dir);
    } else {
        log!("Creating directory {:?}", lib_dir);
        fs::create_dir(lib_dir.clone()).unwrap();
    }

    let library_path = lib_dir.join(format!("lib{}.so", LIBRARY));
    log_var!(library_path);

    let target_path = &TARGET.replace(":", "/");
    log_var!(target_path);

    if !Path::new(&source.join(".git")).exists() {
        run("git", |command| {
            command.arg("clone")
            //.arg(format!("--branch={}", TAG))
                .arg(REPOSITORY)
                .arg(&source)
        });
    }

    log!("cloned the repository to {:?} succesfully.", &source);

    // only configure once, drop a file on the first configure and then
    // skip configuration if the file exists.
    let configure_hint_file_pb = &source.join(".rust-configured");
    let configure_hint_file = Path::new(configure_hint_file_pb);
    if !configure_hint_file.exists() {
        run("bash", |command| {
            command.current_dir(&source)
                .arg("-c")
                .arg("yes''|./autogen.sh")
        });

        run("bash", |command| {
            command.current_dir(&source)
                .arg("-c")
                .arg("yes''|./configure")
        });
        File::create(configure_hint_file).unwrap();
    }

    // make install, this installs it to /usr/local/lib
    run("sudo", |command| {
        command.current_dir(&source)
            .arg("make")
            .arg("install")
    });

    // Clean up after as sudo takes ownership over the files and breaks 
    // cargo clean
    run("sudo", |command| {
        command.current_dir(&source)
            .arg("make")
            .arg("clean")
    });
}

/// Use bindgen to generate the rust code required to interact with the C header file
fn generate_bindings() {
    let out_path = PathBuf::from(&get!("OUT_DIR"));

    if !out_path.join("bindings.rs").exists() {
        log!("Attempting to generate bindings via bindgen.");
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .whitelisted_function(r"readstat_[a-z0-9_]+")
            .whitelisted_type(r"readstat_[a-z]+_t")
            // .whitelisted_type("readstat_error_t")
            // .whitelisted_type("readstat_metadata_t")
            .whitelisted_var(r"READSTAT_HANDLER_[A-Z]+")
            // .whitelisted_var("READSTAT_HANDLER_ABORT")
            // .whitelisted_var("READSTAT_HANDLER_SKIP_VARIABLE")
            .generate()
            .expect("Unable to generate bindings");
        log!("Bindings generated bindings.");

        log!("Attempting to write to file {:?}", out_path.join("bindings.rs"));

        bindings.write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");

        log!("Successfully written bindings to {:?}",
             out_path.join("bindings.rs"));
    } else {
        log!("Bindings already generated. Skipping.");
    }
}

/// Build and run commands with a sane syntax
fn run<F>(name: &str, mut configure: F)
where F: FnMut(&mut Command)-> &mut Command {
    let mut command = Command::new(name);
    let configured = configure(&mut command);

    log!("Executing {:?}", configured);
    if !ok!(configured.status()).success() {
        panic!("Failed to execute {:?}", configured);
    }
    log!("Command {:?} finished successfully.", configured);
}

