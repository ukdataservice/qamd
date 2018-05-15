
use Context;
use report::Variable;
use check::common::contains;

use std::os::raw::c_void;

/// Variable checks

pub fn check_odd_characters(variable: Variable,
                            ctx: *mut c_void) {
    unsafe {
        let context = ctx as *mut Context;

        if let Some(ref config_odd_characters) = (*context).config
            .variable_config
            .odd_characters {
            if contains(&variable.name, config_odd_characters) ||
                contains(&variable.label, config_odd_characters) {

                if (*context).report.variable_checks.odd_characters.is_none() {
                    (*context).report
                        .variable_checks
                        .odd_characters = Some(vec!());
                }

                if let Some(ref mut odd_characters_vec) = (*context)
                        .report
                        .variable_checks
                        .odd_characters {
                    odd_characters_vec.push(variable);
                }
            }
        }
    }
}

