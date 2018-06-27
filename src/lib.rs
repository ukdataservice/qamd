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
extern crate serde_derive;
extern crate serde;
// extern crate serde_json;

extern crate pbr;

#[macro_use]
pub mod macros;

pub mod config;
pub mod report;
pub mod bindings;

mod check;

use self::config::Config;

use self::report::Report;
use self::report::{ Variable, Value };
use self::report::anyvalue::AnyValue;
use self::report::missing::Missing;

use self::check::Check;

use self::bindings::*;

use std::collections::HashMap;

use std::os::raw::{ c_void, c_char, c_int, c_double };
use std::ffi::{ CString, CStr };
use std::io;

use std::clone::Clone;

use pbr::ProgressBar;

pub struct Context {
    config: Config,
    report: Report,
    checks: Check,
    pb: Option<ProgressBar<io::Stdout>>,
    pub variables: Vec<Variable>,
    pub value_labels: HashMap<String, HashMap<String, String>>,
    pub frequency_table: HashMap<Variable, HashMap<Value, i32>>,
}

/// Fuzzy reader, determines file type by the extention
pub fn read(path: &str, config: &Config) -> Result<Report, io::Error> {
    return match (path.ends_with(".dta"),
                  path.ends_with(".sav"),
                  path.ends_with(".por"),
                  path.ends_with(".sas7bdat")) {
        (true, _, _, _) => read_dta(path, config),
        (_, true, _, _) => read_sav(path, config),
        (_, _, true, _) => read_por(path, config),
        (_, _, _, true) => read_sas7bdat(path, config),
        _ => Err(io::Error::new(io::ErrorKind::Other,
                                format!("Failed to determine file type of: {}",
                                    path))),
    };
}

/// Read Stata
pub fn read_dta(path: &str, config: &Config) -> Result<Report, io::Error> {
    return unsafe {
        _read(path, config, readstat_parse_dta)
    };
}

/// Read SPSS
pub fn read_sav(path: &str, config: &Config) -> Result<Report, io::Error> {
    return unsafe {
        _read(path, config, readstat_parse_sav)
    };
}

/// Read SPSS (older format)
pub fn read_por(path: &str, config: &Config) -> Result<Report, io::Error> {
    return unsafe {
        _read(path, config, readstat_parse_por)
    };
}

/// Read SAS
pub fn read_sas7bdat(path: &str, config: &Config)
    -> Result<Report, io::Error> {

    return unsafe {
        _read(path, config, readstat_parse_sas7bdat)
    };
}

/// Parser function type signature
type ParseFn =
    unsafe extern "C" fn(parser: *mut readstat_parser_t,
                         path: *const c_char,
                         user_ctx: *mut c_void) -> readstat_error_t;

/// Read the file using a given ParseFn
unsafe fn _read(path: &str,
                config: &Config,
                file_parser: ParseFn) -> Result<Report, io::Error> {

    let context: *mut Context = Box::into_raw(Box::new(Context {
        config: (*config).clone(),
        report: Report::new(),
        checks: Check::new(),
        pb: None,
        frequency_table: HashMap::new(),
        value_labels: HashMap::new(),
        variables: vec!(),
    }));

    // init the progress bar here
    if let Some(include_progress) = config.progress {
        if include_progress {
            println!("progress bar!");
            (*context).pb = Some(ProgressBar::new(100));
            if let Some(ref mut pb) = (*context).pb {
                pb.format("[=>]");
            }
        }
    }

    let parser: *mut readstat_parser_t = readstat_parser_init();

    readstat_set_metadata_handler(parser, Some(metadata_handler));
    readstat_set_variable_handler(parser, Some(variable_handler));
    readstat_set_value_handler(parser, Some(value_handler));
    readstat_set_value_label_handler(parser, Some(value_label_handler));
    readstat_set_progress_handler(parser, Some(progress_handler));

    let path_to_file = str_to_ptr!(path);
    let error = file_parser(parser, path_to_file, context as *mut c_void);

    readstat_parser_free(parser);

    if let Some(ref mut pb) = (*context).pb {
        pb.finish_print("");
    }

    for check in &(*context).checks.post {
        check(&(*context),
              &(*context).config,
              &mut (*context).report);
    }

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

    // compression
    use readstat_compress_t::*;
    (*context).report.metadata.compression =
        match readstat_get_compression(metadata) {
        READSTAT_COMPRESS_NONE => "None",
        READSTAT_COMPRESS_ROWS => "Rows",
        READSTAT_COMPRESS_BINARY => "Binary",
    }.to_string();

    // dta has no file encoding
    if readstat_get_file_encoding(metadata) != std::ptr::null() {
        (*context).report.metadata.file_encoding = Some(ptr_to_str!(readstat_get_file_encoding(metadata)));
    } else {
        (*context).report.metadata.file_encoding = None;
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Variable callback
unsafe extern "C" fn variable_handler(index: c_int,
                                      variable: *mut readstat_variable_t,
                                      val_labels: *const c_char,
                                      ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    let variable_name = ptr_to_str!(readstat_variable_get_name(variable));

    let label = if readstat_variable_get_label(variable) != std::ptr::null() {
        ptr_to_str!(readstat_variable_get_label(variable))
    } else {
        "".to_string()
    };

    let value_labels = if val_labels != std::ptr::null() {
        ptr_to_str!(val_labels)
    } else {
        "".into()
    };

    let var = Variable {
        // index is zero based, add one to make it human usable
        index: index as i32 + 1,
        name: variable_name,
        label: label,
        value_labels: value_labels,
    };

    (*context).variables.push(var.clone());

    for check in (*context).checks.variable.iter() {
        check(&var,
              &(*context).config,
              &mut (*context).report);
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Value callback
unsafe extern "C" fn value_handler(obs_index: c_int,
                                   variable: *mut readstat_variable_t,
                                   value: readstat_value_t,
                                   ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    let var_index = readstat_variable_get_index(variable);
    let anyvalue = AnyValue::from(value);

    use Missing::*;

    // determine the MISSINGESS
    let missing: Missing = match (
        readstat_value_is_system_missing(value),
        readstat_value_is_tagged_missing(value),
        readstat_value_is_defined_missing(value, variable)) {
        (0, 0, 0) => NOT_MISSING,
        (_, 1, _) => TAGGED_MISSING(readstat_value_tag(value) as u8 as char),
        (_, _, 1) => DEFINED_MISSING,
        (1, _, _) => SYSTEM_MISSING,
        _            => panic!("default case hit"),
    };

    let label: String = if let Some(variable) = (*context).variables.iter().nth(var_index as usize) {
        if let Some(map) = (*context).value_labels.get_mut(&variable.value_labels) {
            map.get(&format!("{}", anyvalue)).unwrap_or(&"".to_string()).to_string()
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let value = Value {
        var_index: var_index + 1,
        row: obs_index + 1,
        value: anyvalue,
        label: label,
        missing: missing,
    };

    // build the frequency table as we collect the values
    if let Some(variable) = (*context).variables.iter().nth(var_index as usize) { // get the variable
        if let Some(ref mut value_occurence_map) = (*context).frequency_table.get_mut(variable) { // check it has been pushed
            if let Some(occurrence) = value_occurence_map.get_mut(&value) {
                (*occurrence) += 1; // already exists
            } else {
                // variable exists, first encounter with this value
                match (*context).frequency_table.get_mut(variable) {
                    Some(val_occ_map) => val_occ_map.insert(value.clone(), 1),
                    None => None,
                };
            }
        } else {
            // no variable found, first encounter with this variable and value
            let mut map: HashMap<Value, i32> = HashMap::new();

            map.insert(value.clone(), 1);
            (*context).frequency_table.insert(variable.clone(), map);
        }
    }

    for check in (*context).checks.value.iter() {
        check(&value,
              &(*context).config,
              &mut (*context).report);
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Value label callback
unsafe extern "C" fn value_label_handler(val_labels: *const c_char,
                                         value: readstat_value_t,
                                         label: *const c_char,
                                         ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    let value_label_id = ptr_to_str!(val_labels);

    let value_str: String = format!("{}", AnyValue::from(value));

    if !(*context).value_labels.contains_key(&value_label_id) {
        (*context).value_labels.insert(value_label_id.clone(), HashMap::new());
    }

    if let Some(map) = (*context).value_labels.get_mut(&value_label_id) {
        (*map).insert(value_str, ptr_to_str!(label));
    }

    return READSTAT_HANDLER_OK as c_int;
}

unsafe extern "C" fn progress_handler(progress: c_double,
                                      ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    if let Some(ref mut pb) = (*context).pb {
        pb.set((progress * 100.0) as u64);
    }

    return READSTAT_HANDLER_OK as c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;
    use self::config::{VariableConfig, ValueConfig};

    fn setup() -> Config {
        Config {
            primary_variable: None,
            disclosive_outliers: None,
            variable_config: VariableConfig {
                odd_characters: None,
                missing_variable_labels: Some(config::Setting {
                    setting: true,
                    file_types: vec!(config::FileType::SAV,
                                     config::FileType::DTA,
                                     config::FileType::SAS7BDAT)
                }),
                label_max_length: None,
            },
            value_config: ValueConfig {
                odd_characters: None,
                system_missing_value_threshold: None,
                defined_missing_no_label: None,
                label_max_length: None,
            },
        }
    }

    #[test]
    fn test_read_dta() {
        let config = setup();

        let report = ok!(read_dta("test/mtcars.dta", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn test_read_sav() {
        let config = setup();

        let report = ok!(read_sav("test/mtcars.sav", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn test_read_sas7bdat() {
        let config = setup();

        let report = ok!(read_sas7bdat("test/mtcars.sas7bdat", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn reader_should_error_on_enoent() {
        let config = setup();

        let err = match read_dta("", &config) {
            Ok(_) => "this should never be run".to_string(),
            Err(e) => e.description().to_string()
        };

        assert_eq!(err, "Unable to open file");
    }
}
