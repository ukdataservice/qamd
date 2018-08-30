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
extern crate csv;

#[macro_use]
pub mod macros;

pub mod config;
pub mod report;
pub mod readstat;
pub mod html;

mod check;

