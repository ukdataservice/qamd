
use Context;
use report::Value;
use check::ValueCheckFn;
use check::common::contains;

use std::os::raw::c_void;

// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec!(check_odd_characters)
}

// Value checks

/// Check for odd characters in the value and value label.
/// If a value is determined to contain any odd character(s), the value is pushed to the report.
fn check_odd_characters(value: &Value, ctx: *mut c_void) {
    unsafe {
        let context = ctx as *mut Context;

        if let Some(ref config_odd_characters) = (*context).config
            .value_config
            .odd_characters {

            if contains(&format!("{}", &value.value), config_odd_characters) ||
                contains(&value.label, config_odd_characters) {

                if (*context).report
                    .value_checks
                    .odd_characters.is_none() {
                        (*context).report
                            .value_checks
                            .odd_characters = Some(vec!());
                }

                if let Some(ref mut odd_characters_vec) = (*context)
                    .report
                    .value_checks
                    .odd_characters {
                            odd_characters_vec.push(value.clone());
                }
            }
        }
    }
}

