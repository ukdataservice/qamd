use model::anyvalue::AnyValue;
use model::missing::Missing;
use model::value::Value;
use model::variable::Variable;

use readstat::bindings::*;
use readstat::context::Context;

use chrono::naive::NaiveDateTime;

use std::collections::HashMap;

use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int, c_void};

use std::ptr;

/// Process file metadata
pub unsafe extern "C" fn metadata_handler(
    metadata: *mut readstat_metadata_t,
    ctx: *mut c_void,
) -> c_int {
    let context = ctx as *mut Context;

    (*context).report.metadata.raw_case_count = readstat_get_row_count(metadata);
    (*context).report.metadata.variable_count = readstat_get_var_count(metadata);

    //(*context).report.metadata.creation_time = readstat_get_creation_time(metadata);
    //(*context).report.metadata.modified_time = readstat_get_modified_time(metadata);
    let creation_time: i64 = readstat_get_creation_time(metadata);
    let modified_time: i64 = readstat_get_modified_time(metadata);

    (*context).report.metadata.creation_time = NaiveDateTime::from_timestamp(creation_time, 0);
    (*context).report.metadata.modified_time = NaiveDateTime::from_timestamp(modified_time, 0);

    (*context).report.metadata.file_label = ptr_to_str!(readstat_get_file_label(metadata));
    (*context).report.metadata.file_format_version =
        readstat_get_file_format_version(metadata) as i64;

    // compression
    use self::readstat_compress_t::*;
    (*context).report.metadata.compression = match readstat_get_compression(metadata) {
        READSTAT_COMPRESS_NONE => "None",
        READSTAT_COMPRESS_ROWS => "Rows",
        READSTAT_COMPRESS_BINARY => "Binary",
    }
    .to_string();

    // dta has no file encoding
    if readstat_get_file_encoding(metadata) != ptr::null() {
        (*context).report.metadata.file_encoding =
            Some(ptr_to_str!(readstat_get_file_encoding(metadata)));
    } else {
        (*context).report.metadata.file_encoding = None;
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Variable callback
pub unsafe extern "C" fn variable_handler(
    index: c_int,
    variable: *mut readstat_variable_t,
    val_labels: *const c_char,
    ctx: *mut c_void,
) -> c_int {
    let context = ctx as *mut Context;

    let var = Variable::from_raw_parts(variable, val_labels);
    assert_eq!(var.index, index as i32);

    (*context).variables.push(var.clone());
    for check in (*context).checks.variable.iter() {
        check(&var, &(*context).config, &mut (*context).report);
    }

    // data type occurences, count the number of text & numeric variables
    if let Some(occ) = (*context).report.metadata.data_type_occurrences.get_mut(&var.type_) {
        (*occ) += 1;
    } else {
        (*context).report.metadata.data_type_occurrences.insert(var.type_, 1);
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Value callback
pub unsafe extern "C" fn value_handler(
    obs_index: c_int,
    variable: *mut readstat_variable_t,
    value: readstat_value_t,
    ctx: *mut c_void,
) -> c_int {
    let context = ctx as *mut Context;

    let var = (*context)
        .variables
        .iter()
        .nth(readstat_variable_get_index(variable) as usize)
        .unwrap();
    let anyvalue = AnyValue::from(value);

    // determine the MISSINGESS
    let missing: Missing = match (
        readstat_value_is_system_missing(value),
        readstat_value_is_tagged_missing(value),
        readstat_value_is_defined_missing(value, variable),
    ) {
        (0, 0, 0) => Missing::NOT_MISSING,
        (_, 1, _) => Missing::TAGGED_MISSING(readstat_value_tag(value) as u8 as char),
        (_, _, 1) => Missing::DEFINED_MISSING,
        (1, _, _) => Missing::SYSTEM_MISSING,
        _ => panic!("default case hit"),
    };

    let label: String = if let Some(map) = (*context).value_labels.get_mut(&var.value_labels) {
        map.get(&format!("{}", anyvalue))
            .unwrap_or(&"".to_string())
            .to_string()
    } else {
        "".to_string()
    };

    let value = Value {
        variable: var.clone(),
        row: obs_index,
        value: anyvalue,
        label: label,
        missing: missing,
    };

    // build the frequency table as we collect the values
    if let Some(ref mut value_occurence_map) = (*context).frequency_table.get_mut(&var) {
        // check it has been pushed
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
        check(&value, &(*context).config, &mut (*context).report);
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Value label callback
pub unsafe extern "C" fn value_label_handler(
    val_labels: *const c_char,
    value: readstat_value_t,
    label: *const c_char,
    ctx: *mut c_void,
) -> c_int {
    let context = ctx as *mut Context;

    let value_label_id = ptr_to_str!(val_labels);

    let value_str: String = format!("{}", AnyValue::from(value));

    if !(*context).value_labels.contains_key(&value_label_id) {
        (*context)
            .value_labels
            .insert(value_label_id.clone(), HashMap::new());
    }

    if let Some(map) = (*context).value_labels.get_mut(&value_label_id) {
        (*map).insert(value_str, ptr_to_str!(label));
    }

    return READSTAT_HANDLER_OK as c_int;
}

pub unsafe extern "C" fn progress_handler(progress: c_double, ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    if let Some(ref mut pb) = (*context).pb {
        pb.set((progress * 100.0) as u64);
    }

    return READSTAT_HANDLER_OK as c_int;
}
