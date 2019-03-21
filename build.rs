extern crate bindgen;
extern crate curl;
extern crate pkg_config;

use curl::easy::Easy;

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::path::PathBuf;
use std::env;

const LIBRARY: &'static str = "readstat";
const LIB_SEARCH_PATH: &'static str = "readstat/lib/static/";

const READSTAT_URL: &'static str = "https://github.com/WizardMac/ReadStat/archive/master.zip";

macro_rules! get(($name:expr) => (env::var($name).unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("libreadstat/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("libreadstat/build.rs:{}: ", $fmt),
                                          line!(), $($arg)*));
}
// macro_rules! log_var(($var:ident) =>
//                      (log!(concat!(stringify!($var), " = {:?}"), $var)));

fn main() {
    // if pkg_config::find_library(LIBRARY).is_ok() {
    //     log!("Returning early because {} was already found", LIBRARY);
    //     return;
    // }

    println!("cargo:rustc-link-lib=static={}", LIBRARY);
    println!("cargo:rustc-link-search={}", LIB_SEARCH_PATH);

    get_readstat();

    // build_readstat(); // ???

    generate_bindings();
}

/// Use bindgen to generate the rust code required to interact with the C header file
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

fn get_readstat() {
    let out_path = PathBuf::from(&get!("OUT_DIR"));
    let tmp_file_path = out_path.join("readstat_master.zip");
    let tmp_file_path_clone = tmp_file_path.clone();

    // Dowload zip
    if !&tmp_file_path.exists() {
        let mut easy = Easy::new();

        let _ = easy.url(READSTAT_URL);
        easy.write_function(move |data| {
            let tmp_file = File::create(&tmp_file_path_clone).unwrap();
            let mut buffer = BufWriter::new(tmp_file);

            Ok(buffer.write(data).unwrap())
        }).unwrap();
        log!("Downloading: {}", &READSTAT_URL);
        easy.perform().unwrap();
        log!("Donloaded: {} to {:?}", &READSTAT_URL, &tmp_file_path);
    }
}

