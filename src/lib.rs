//!
//! # Overview
//!
//! Rust only QAMyData. Uses
//! [ReadStat](https://github.com/WizardMac/ReadStat) C library, with thanks
//! to WizardMac.
//!
//! # Examples
//! ```
//! println!("Hello, World!");
//! assert!(4 == 2 + 2);
//! ```
//!

#[macro_use]
extern crate horrorshow;

#[macro_use]
extern crate serde_derive;
extern crate serde;
// extern crate serde_json;

extern crate pbr;

#[macro_use]
pub mod macros;

pub mod config;
pub mod report;
pub mod readstat;
pub mod html;

mod check;

//
// use self::config::Config;
//
// use self::report::Report;
// use self::report::{ Variable, Value };
// use self::report::anyvalue::AnyValue;
// use self::report::missing::Missing;
//
// use self::check::Check;
//
// use self::bindings::*;
//
// use std::collections::HashMap;
//
// use std::os::raw::{ c_void, c_char, c_int, c_double };
// use std::ffi::{ CString, CStr };
// use std::io;
// use std::path::Path;
//
// use std::clone::Clone;
//
// use pbr::ProgressBar;
//

