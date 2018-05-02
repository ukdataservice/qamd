//!
//! # Overview
//!
//! Rust only QAMyData. Uses
//! [ReadStat](https://github.com/WizardMac/ReadStat) C library, with thanks to
//! WizardMac.
//!
//! # Examples
//! ```
//! ```
//!

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate serde_derive;

extern crate serde;
// extern crate serde_json;

pub mod config;
pub mod report;

use self::config::Config;
use self::report::{Report, Metadata};

use std::os::raw::{c_int, c_void, c_char};
use std::ffi::{CString, CStr};
use std::io;

use std::clone::Clone;

macro_rules! str_to_ptr(($name:expr) =>
                        (ok!(CString::new($name)).into_raw()));
macro_rules! ptr_to_str(($name:expr) =>
                        (CStr::from_ptr($name).to_string_lossy().into_owned()));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));

#[derive(Debug)]
struct Context {
    config: Config,
    report: Report
}

/// Read Stata
pub fn read_dta(path: &str) -> Result<Report, io::Error> {
    return unsafe {
        read(path, readstat_parse_dta)
    };
}

/// Read SPSS
pub fn read_sav(path: &str) -> Result<Report, io::Error> {
    return unsafe {
        read(path, readstat_parse_sav)
    };
}

/// Read SPSS (older format)
pub fn read_por(path: &str) -> Result<Report, io::Error> {
    return unsafe {
        read(path, readstat_parse_por)
    };
}

/// Read SAS
pub fn read_sas7bdat(path: &str) -> Result<Report, io::Error> {
    return unsafe {
        read(path, readstat_parse_sas7bdat)
    };
}

/// Parser function type signature
type GenericParseFn =
    unsafe extern "C" fn(parser: *mut readstat_parser_t,
                         path: *const c_char,
                         user_ctx: *mut c_void) -> readstat_error_t;

/// Read the file using a given GenericParseFn
unsafe fn read(path: &str, file_parser: GenericParseFn)
               -> Result<Report, io::Error> {
    let context: *mut Context = Box::into_raw(Box::new(Context {
        config: Config {
            file_encoding: true,
            odd_characters: vec!(),
            missing_variable_labels: true,
            system_missing_value_threshold: Some(25),
        },
        report: Report {
            odd_characters: None,
            metadata: Metadata {
                raw_case_count: 0,
                case_count: None,
                variable_count: 0,
                creation_time: 0,
                modified_time: 0,
                file_label: "".into(),
                file_format_version: 0,
                file_encoding: None,
            }
        },
    }));

    let parser: *mut readstat_parser_t = readstat_parser_init();

    readstat_set_metadata_handler(parser, Some(metadata_handler));
    readstat_set_variable_handler(parser, Some(variable_handler));
    readstat_set_value_handler(parser, Some(value_handler));
    readstat_set_value_label_handler(parser, Some(value_label_handler));

    let path_to_file = str_to_ptr!(path);
    let error = file_parser(parser, path_to_file, context as *mut c_void);

    readstat_parser_free(parser);

    if error != readstat_error_t::READSTAT_OK {
        Err(handle_error(error))
    } else {
        Ok((*context).report.clone())
    }
}

/// Create an error object from a readstat error
fn handle_error(error: readstat_error_t) -> io::Error {
    unsafe {
        io::Error::new(io::ErrorKind::Other,
                       ptr_to_str!(readstat_error_message(error)))
    }
}

/// Process file metadata
unsafe extern "C" fn metadata_handler(metadata: *mut readstat_metadata_t,
                                      ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    (*context).report.metadata.raw_case_count = readstat_get_row_count(metadata);
    (*context).report.metadata.variable_count = readstat_get_var_count(metadata);

    (*context).report.metadata.creation_time = readstat_get_creation_time(metadata);
    (*context).report.metadata.modified_time = readstat_get_modified_time(metadata);

    (*context).report.metadata.file_label = ptr_to_str!(readstat_get_file_label(metadata));
    (*context).report.metadata.file_format_version = readstat_get_file_format_version(metadata) as i64;

    // dta has no file encoding
    if readstat_get_file_encoding(metadata) != std::ptr::null() {
        (*context).report.metadata.file_encoding = Some(ptr_to_str!(readstat_get_file_encoding(metadata)));
    } else {
        (*context).report.metadata.file_encoding = None;
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Variable callback
unsafe extern "C" fn variable_handler(_index: c_int,
                                      variable: *mut readstat_variable_t,
                                      val_labels: *const c_char,
                                      ctx: *mut c_void) -> c_int {
    // let context = ctx as *mut DataFrame;

    // let variable_name = ptr_to_str!(readstat_variable_get_name(variable));

    // let label = if readstat_variable_get_label(variable) != std::ptr::null() {
    //     ptr_to_str!(readstat_variable_get_label(variable))
    // } else {
    //     "".to_string()
    // };

    // let val_labels_string = if val_labels != std::ptr::null() {
    //     ptr_to_str!(val_labels)
    // } else {
    //     "".to_string()
    // };

    // let var = Variable::new(variable_name, label, val_labels_string);
    // (*context).values
    //     .insert(var.clone(), vec!());
    // (*context).frequency_table
    //     .insert(var, HashMap::new());

    return READSTAT_HANDLER_OK as c_int;
}

/// Value callback
unsafe extern "C" fn value_handler(_obs_index: c_int,
                                   variable: *mut readstat_variable_t,
                                   value: readstat_value_t,
                                   ctx: *mut c_void) -> c_int {
    // let context = ctx as *mut DataFrame;
    // //let var_index = readstat_variable_get_index(variable);
    // let var_name = ptr_to_str!(readstat_variable_get_name(variable));
    // let key = (*context).values
    //     .keys()
    //     .find(|&k| {k.name == var_name})
    //     .unwrap();

    // let value_as_any_value: AnyValue = AnyValue::from(value);

    // // if !(*context).values.contains_key(&key) {
    // //     println!("Warn: Key missing: {:?}", key);
    // // }

    // use Missing::*;

    // // determine the MISSINGESS
    // let missing: Missing = match (
    //                               readstat_value_is_system_missing(value),
    //                               readstat_value_is_tagged_missing(value),
    //                               readstat_value_is_defined_missing(value, variable)) {
    //     (0, 0, 0) => NOT_MISSING,
    //     (_, 1, _) => TAGGED_MISSING(readstat_value_tag(value) as u8 as char),
    //     (_, _, 1) => DEFINED_MISSING,
    //     (1, _, _) => SYSTEM_MISSING,
    //     _            => panic!("default case hit"),
    // };

    // let new_value = Value::new(value_as_any_value, missing);

    // let value_vec = (*context).values.get_mut(&key).unwrap();
    // value_vec.push(new_value.clone());

    // let frequency_table_map = (*context).frequency_table.get_mut(&key).unwrap();

    // if frequency_table_map.contains_key(&new_value) {
    //     let count = frequency_table_map.get_mut(&new_value).unwrap();
    //     (*count) += 1;
    // } else {
    //     frequency_table_map.insert(new_value, 1);
    // }

    return READSTAT_HANDLER_OK as c_int;
}

/// Value label callback
unsafe extern "C" fn value_label_handler(val_labels: *const c_char,
                                         value: readstat_value_t,
                                         label: *const c_char,
                                         ctx: *mut c_void) -> c_int {
    // let context = ctx as *mut DataFrame;

    // use readstat_type_t::*;

    // let mut value_str: String = match readstat_value_type(value) {
    //     READSTAT_TYPE_STRING =>
    //         ptr_to_str!(readstat_string_value(value)),
    //     READSTAT_TYPE_INT8 =>
    //         (readstat_int8_value(value) as i8).to_string(),
    //     READSTAT_TYPE_INT16 =>
    //         (readstat_int16_value(value) as i16).to_string(),
    //     READSTAT_TYPE_INT32 =>
    //         (readstat_int32_value(value) as i32).to_string(),
    //     READSTAT_TYPE_FLOAT =>
    //         format!("{:?}", readstat_float_value(value)),
    //     READSTAT_TYPE_DOUBLE =>
    //         format!("{:?}", readstat_double_value(value)),
    //     READSTAT_TYPE_STRING_REF =>
    //         "REF TYPE".to_string(),
    // };

    // // hack to make the decimal point show up.
    // if !value_str.contains(".") {
    //     value_str += ".0";
    // }

    // let key = if val_labels != std::ptr::null() {
    //     ptr_to_str!(val_labels)
    // } else {
    //     "".to_string()
    // };

    // if !(*context).value_label_dict.contains_key(&key) {
    //     (*context).value_label_dict.insert(key.clone(), HashMap::new());
    // }

    // (*context).value_label_dict.get_mut(&key)
    //     .unwrap()
    //     .insert(value_str.clone(), ptr_to_str!(label));

    // // if &key == "labels0" {
    // //     println!("{}: {{ {}: {} }}", &key, value_str, ptr_to_str!(label));
    // // }

    return READSTAT_HANDLER_OK as c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    #[test]
    fn test_read_dta() {
        let data = read_dta("test/mtcars.dta").unwrap();
        assert_eq!(data.var_count, 12);
        assert_eq!(data.row_count, 32);
    }

    #[test]
    fn test_read_sav() {
        let data = read_sav("test/mtcars.sav").unwrap();
        assert_eq!(data.var_count, 12);
        assert_eq!(data.row_count, 32);
    }

    #[test]
    fn test_tead_sas7bdat() {
        let data = read_sas7bdat("test/mtcars.sas7bdat").unwrap();
        assert_eq!(data.var_count, 12);
        assert_eq!(data.row_count, 32);
    }

    #[test]
    fn test_read_err() {
        let err = match read_dta("") {
            Ok(_) => "failed".to_string(),
            Err(e) => e.description().to_string()
        };

        println!("{:?}", err);
        assert_eq!(err, "Unable to open file");
    }
}
