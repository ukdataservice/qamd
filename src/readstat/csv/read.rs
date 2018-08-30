
use readstat::bindings::*;
use csv;
use pbr::ProgressBar;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;
// use std::error::Error;
use std::path::Path;
use std::ffi::CStr;

use config::Config;
use report::Report;
use readstat::context::Context;
use check::Check;

struct ParsedCSV {
    pub headers: Vec<String>,
    pub records: HashMap<String, csv::StringRecord>,
}

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
    let error = readstat_error_t::READSTAT_OK;

    if let Some(ref mut pb) = (*context).pb {
        pb.finish_print("");
    }

    if error != readstat_error_t::READSTAT_OK {
        Err(io::Error::new(io::ErrorKind::Other,
                           ptr_to_str!(readstat_error_message(error))))
    } else {
        // post checks
        for check in &(*context).checks.post {
            check(&(*context),
                  &(*context).config,
                  &mut (*context).report);
        }

        Ok((*context).report.clone())
    }
}

fn get_file_contents(path: &str) -> io::Result<String> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();

    buf_reader.read_to_string(&mut contents)?;

    Ok(contents)
}

// fn parse_csv(contents: String) -> Result<ParsedCSV, Box<Error>> {
//     let mut rdr = csv::reader::from_reader(contents.as_bytes());
// 
//     let mut parsed = ParsedCSV {
//         headers: vec!(),
//         records: HashMap::new(),
//     };
// 
//     for (row_index, result) in rdr.records().enumerate() {
//         let record: csv::StringRecord = result?;
//         parsed.headers.push(header);
//     }
// 
//     let headers: csv::StringRecord = rdr.headers()?.clone();
//     for (column_index, (field, header)) in rdr.records().iter()
//         .zip(headers.iter())
//         .enumerate() {
//         parsed.records.push(record);
//     }
// 
//     Ok(parsed)
// }

fn as_fixed_size(a: &mut [i8], s: &mut String) {
    let bytes = unsafe { s.as_mut_vec() };

    for (first, second) in a.iter_mut().zip(bytes) {
        *first = *second as i8;
    }
}

