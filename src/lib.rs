//!
//! # Overview
//!
//! Rust only QAMyData. Uses
//! [ReadStat](https://github.com/WizardMac/ReadStat) C library, with thanks
//! to WizardMac.
//!
//! The core library revolves arround a forigen function interface (FFI) with
//! the ReadStat C library. From here the check module defines the checks to
//! be carried out and at what stage.
//!

#[macro_use]
extern crate horrorshow;

#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate csv as csv_crate;
extern crate pbr;
extern crate regex;

#[macro_use]
pub mod macros;

pub mod config;
pub mod html;
pub mod model;
pub mod readstat;
pub mod report;

mod check;
