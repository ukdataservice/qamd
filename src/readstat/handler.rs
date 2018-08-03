
use report::{ Value, Variable };
use report::missing::Missing;
use report::anyvalue::AnyValue;

use readstat::bindings::*;
use readstat::context::Context;

use std::collections::HashMap;

use std::os::raw::{ c_void, c_char, c_int, c_double };
use std::ffi::{ /*CString,*/ CStr };

use std::ptr;

/// Process file metadata
pub unsafe extern "C" fn metadata_handler(metadata: *mut readstat_metadata_t,
                                      ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    (*context).report.metadata.raw_case_count = readstat_get_row_count(metadata);
    (*context).report.metadata.variable_count = readstat_get_var_count(metadata);

    (*context).report.metadata.creation_time = readstat_get_creation_time(metadata);
    (*context).report.metadata.modified_time = readstat_get_modified_time(metadata);

    (*context).report.metadata.file_label = ptr_to_str!(readstat_get_file_label(metadata));
    (*context).report.metadata.file_format_version = readstat_get_file_format_version(metadata) as i64;

    // compression
    use self::readstat_compress_t::*;
    (*context).report.metadata.compression =
        match readstat_get_compression(metadata) {
        READSTAT_COMPRESS_NONE => "None",
        READSTAT_COMPRESS_ROWS => "Rows",
        READSTAT_COMPRESS_BINARY => "Binary",
    }.to_string();

    // dta has no file encoding
    if readstat_get_file_encoding(metadata) != ptr::null() {
        (*context).report.metadata.file_encoding = Some(ptr_to_str!(readstat_get_file_encoding(metadata)));
    } else {
        (*context).report.metadata.file_encoding = None;
    }

    return READSTAT_HANDLER_OK as c_int;
}

/// Variable callback
pub unsafe extern "C" fn variable_handler(index: c_int,
                                          variable: *mut readstat_variable_t,
                                          val_labels: *const c_char,
                                          ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    let variable_name = ptr_to_str!(readstat_variable_get_name(variable));

    let label = if readstat_variable_get_label(variable) != ptr::null() {
        ptr_to_str!(readstat_variable_get_label(variable))
    } else {
        String::new()
    };

    let value_format = if readstat_variable_get_format(variable) != ptr::null() {
        ptr_to_str!(readstat_variable_get_format(variable))
    } else {
        String::new()
    };

    let value_labels = if val_labels != ptr::null() {
        ptr_to_str!(val_labels)
    } else {
        "".into()
    };

    let var = Variable {
        // index is zero based, add one to make it human usable
        index: index as i32 + 1,
        name: variable_name,
        label: label,
        value_format: value_format,
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
pub unsafe extern "C" fn value_handler(obs_index: c_int,
                                   variable: *mut readstat_variable_t,
                                   value: readstat_value_t,
                                   ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    let var_index = readstat_variable_get_index(variable);
    let anyvalue = AnyValue::from(value);


    // determine the MISSINGESS
    let missing: Missing = match (
        readstat_value_is_system_missing(value),
        readstat_value_is_tagged_missing(value),
        readstat_value_is_defined_missing(value, variable)) {
        (0, 0, 0) => Missing::NOT_MISSING,
        (_, 1, _) => Missing::TAGGED_MISSING(readstat_value_tag(value) as u8 as char),
        (_, _, 1) => Missing::DEFINED_MISSING,
        (1, _, _) => Missing::SYSTEM_MISSING,
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
pub unsafe extern "C" fn value_label_handler(val_labels: *const c_char,
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

pub unsafe extern "C" fn progress_handler(progress: c_double,
                                      ctx: *mut c_void) -> c_int {
    let context = ctx as *mut Context;

    if let Some(ref mut pb) = (*context).pb {
        pb.set((progress * 100.0) as u64);
    }

    return READSTAT_HANDLER_OK as c_int;
}

