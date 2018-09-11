
use readstat::bindings::*;
use pbr::ProgressBar;
use csv_crate::Reader;

use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::time;
use std::collections::HashMap;
use std::path::Path;
use std::ffi::CStr;

use config::Config;
use report::{ Report, Variable, Value };
use report::missing::Missing;
use report::anyvalue::AnyValue;
use readstat::context::Context;
use check::Check;

pub unsafe fn read_csv(path: &str, config: &Config) -> Result<Report, io::Error> {
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

    // parse, loop & checks, build context
    let error: readstat_error_t = match get_file_contents(path) {
        Ok(contents) => {
            set_metadata(path, context);
            parse_csv(contents, context)
        },
        Err(_err) => readstat_error_t::READSTAT_ERROR_OPEN,
    };

    if let Some(ref mut pb) = (*context).pb {
        pb.finish_print("");
    }

    if error != readstat_error_t::READSTAT_OK {
        Err(io::Error::new(io::ErrorKind::Other,
                           ptr_to_str!(readstat_error_message(error))))
    } else {
        // post checks
        for check in &(*context).checks.post {
            check(&mut (*context));
        }

        Ok((*context).report.clone())
    }
}

unsafe fn parse_csv(contents: String,
                    context: *mut Context) -> readstat_error_t {
    let mut rdr = Reader::from_reader(contents.as_bytes());

    match rdr.headers() {
        Ok(headers) => {
            for (column_index, variable) in headers.iter().enumerate() {
                let var = Variable {
                    index: column_index as i32,
                    name: variable.to_string(),
                    label: String::new(),
                    value_format: String::new(),
                    value_labels: String::new(),
                };

                for check in &(*context).checks.variable {
                    check(&var,
                          &(*context).config,
                          &mut (*context).report);
                }
                (*context).variables.push(var);
            }
        },
        Err(_) => return readstat_error_t::READSTAT_ERROR_PARSE,
    }

    for (row_index, result) in rdr.records().enumerate() {
        let record = result.unwrap();

        for (column_index, field) in record.iter().enumerate() {
            let var = (*context)
                .variables
                .iter()
                .find(|ref v| {
                    v.index == column_index as i32
                })
                .unwrap();

            let missing: Missing = match field.is_empty() {
                true  => Missing::SYSTEM_MISSING,
                false => Missing::NOT_MISSING,
            };

            let value = Value {
                variable: var.clone(),
                row: row_index as i32,
                value: AnyValue::from(field),
                label: String::new(),
                missing: missing,
            };

            // build the frequency table as we collect the values
            if let Some(ref mut value_occurence_map) = (*context)
                    .frequency_table
                    .get_mut(&var) {
                if let Some(occurrence) = value_occurence_map.get_mut(&value) {
                    (*occurrence) += 1; // already exists
                } else {
                    // variable exists, first encounter with this value
                    match (*context).frequency_table.get_mut(&var) {
                        Some(val_occ_map) => val_occ_map.insert(value.clone(), 1),
                        None => None,
                    };
                }
            } else {
                // variable not found
                // first encounter with this variable and value
                let mut map: HashMap<Value, i32> = HashMap::new();

                map.insert(value.clone(), 1);
                (*context).frequency_table.insert(var.clone(), map);
            }

            for check in (*context).checks.value.iter() {
                check(&value,
                      &(*context).config,
                      &mut (*context).report);
            }
        }
    }

    readstat_error_t::READSTAT_OK
}

unsafe fn set_metadata(path: &str, context: *mut Context) {
    let metadata = fs::metadata(path).unwrap();
    let contents = get_file_contents(path).unwrap();
    let mut rdr = Reader::from_reader(contents.as_bytes());

    (*context).report.metadata.raw_case_count = rdr.records().count() as i32;

    match rdr.headers() {
        Ok(headers) => {
            (*context).report.metadata.variable_count = headers.iter().count() as i32;
        },
        Err(_) => (),
    }

    (*context).report.metadata.creation_time = {
        let sys_time = metadata.created().unwrap_or(time::UNIX_EPOCH);
        sys_time.duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64
    };

    (*context).report.metadata.modified_time = {
        let sys_time = metadata.modified().unwrap();
        sys_time.duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64
    };
}

fn get_file_contents(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

// fn as_fixed_size(a: &mut [i8], s: &mut String) {
//     let bytes = unsafe { s.as_mut_vec() };
// 
//     for (first, second) in a.iter_mut().zip(bytes) {
//         *first = *second as i8;
//     }
// }

