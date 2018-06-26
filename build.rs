extern crate bindgen;
extern crate pkg_config;

use std::path::PathBuf;
use std::env;

const LIBRARY: &'static str = "readstat";
const FRAMEWORK_LIBRARY: &'static str = "readstat_framework";

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

