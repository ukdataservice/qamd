
use report::Report;
use config::Config;
use check::Check;

use readstat::bindings::*;
use readstat::handler::*;
use readstat::context::Context;

use readstat::csv::read::qamd_parse_csv;

use std::collections::HashMap;

use std::os::raw::{ c_void, c_char };
use std::ffi::{ CString, CStr };
use std::io;
use std::path::Path;

use pbr::ProgressBar;

/// Fuzzy reader, determines file type by the extention
pub fn read(path: &str, config: &Config) -> io::Result<Report> {
    return match (path.ends_with(".csv"),
                  path.ends_with(".dta"),
                  path.ends_with(".sav"),
                  path.ends_with(".por"),
                  path.ends_with(".sas7bdat")) {
        (true, _, _, _, _) => read_csv(path, config),
        (_, true, _, _, _) => read_dta(path, config),
        (_, _, true, _, _) => read_sav(path, config),
        (_, _, _, true, _) => read_por(path, config),
        (_, _, _, _, true) => read_sas7bdat(path, config),
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

pub fn read_csv(path: &str, config: &Config) -> Result<Report, io::Error> {
    return unsafe {
        _read(path, config, qamd_parse_csv)
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
            (*context).pb = Some(ProgressBar::new(100));
            if let Some(ref mut pb) = (*context).pb {
                pb.format("[=>]");
            }
        }
    }

    if let Some(file_name) =  Path::new(&path).file_name() {
        (*context).report.metadata.file_name =
            ok!(file_name.to_str()).to_string();
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  "Unable to open file"));
    }

    // init parser & set handlers
    let parser: *mut readstat_parser_t = readstat_parser_init();

    readstat_set_metadata_handler(parser, Some(metadata_handler));
    readstat_set_variable_handler(parser, Some(variable_handler));
    readstat_set_value_handler(parser, Some(value_handler));
    readstat_set_value_label_handler(parser, Some(value_label_handler));
    readstat_set_progress_handler(parser, Some(progress_handler));

    let path_to_file = str_to_ptr!(path);
    let error = file_parser(parser, path_to_file, context as *mut c_void);

    // cleanup
    readstat_parser_free(parser);

    if let Some(ref mut pb) = (*context).pb {
        pb.finish_print("");
    }

    if error != readstat_error_t::READSTAT_OK {
        Err(handle_error(error))
    } else {
        // post checks
        for check in &(*context).checks.post {
            check(&(*context),
                  &(*context).config,
                  &mut (*context).report);
        }

        debug!("{:#?}", *context);

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;

    #[test]
    fn test_read_dta() {
        let config = Config::new();

        let report = ok!(read_dta("test/mtcars.dta", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn test_read_sav() {
        let config = Config::new();

        let report = ok!(read_sav("test/mtcars.sav", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn test_read_sas7bdat() {
        let config = Config::new();

        let report = ok!(read_sas7bdat("test/mtcars.sas7bdat", &config));
        assert_eq!(report.metadata.variable_count, 12);
        assert_eq!(report.metadata.raw_case_count, 32);
    }

    #[test]
    fn reader_should_error_on_enoent() {
        let config = Config::new();

        let err = match read_dta("", &config) {
            Ok(_) => "this should never be run".to_string(),
            Err(e) => e.description().to_string(),
        };

        assert_eq!(err, "Unable to open file");
    }
}
