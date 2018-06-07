
use Context;
use report::Variable;
use check::VariableCheckFn;
use check::common::contains;

use std::os::raw::c_void;

// Register the checks with the context object
pub fn register() -> Vec<VariableCheckFn> {
    vec!(check_label,
         check_label_length,
         check_odd_characters)
}

/// Variable checks

fn check_label(variable: &Variable, ctx: *mut c_void) {
    unsafe {
        let context = ctx as *mut Context;

        if let Some(ref config_missing_variable_labels) = (*context)
            .config
            .variable_config
            .missing_variable_labels
            .setting {

                if *config_missing_variable_labels {
                    if variable.label == "" {
                        if (*context).report.variable_checks.missing_variable_labels.is_none() {
                            (*context).report.variable_checks.missing_variable_labels = Some(vec!());
                        }

                        if let Some(ref mut vars_missing_labels) = (*context).report
                            .variable_checks
                            .missing_variable_labels {
                                vars_missing_labels.push(variable.clone());
                            }
                    }
                }
        }
    }
}

fn check_label_length(variable: &Variable, ctx: *mut c_void) {
    unsafe {
        let context = ctx as *mut Context;

        if let Some(ref label_max_length) = (*context)
            .config
            .variable_config
            .label_max_length
            .setting {

            if variable.label.len() > (*label_max_length) as usize {
                if (*context).report.variable_checks.label_max_length.is_none() {
                    (*context).report.variable_checks.label_max_length = Some(vec!());
                }

                if let Some(ref mut vars_label_max_length) = (*context)
                    .report
                    .variable_checks
                    .label_max_length {

                    vars_label_max_length.push(variable.clone());
                }
            }
        }
    }
}

fn check_odd_characters(variable: &Variable, ctx: *mut c_void) {
    unsafe {
        let context = ctx as *mut Context;

        if let Some(ref config_odd_characters) = (*context).config
            .variable_config
            .odd_characters
            .setting {
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
                    odd_characters_vec.push(variable.clone());
                }
            }
        }
    }
}



