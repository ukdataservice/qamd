
use config::Config;
use report::{ Report, Value, Status, Locator };
use report::missing::Missing;

use check::{ValueCheckFn, contains};

/// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec!(odd_characters,
         label_max_length,
         value_defined_missing_no_label)
}

// Value checks

/// Check for odd characters in the value and value label.
/// If a value is determined to contain any odd character(s),
/// the number of fails (or warns) are incremented.
fn odd_characters(value: &Value,
                  config: &Config,
                  report: &mut Report) {
    if let Some(ref setting) = config.value_config.odd_characters {
        include_check!(report.summary.value_odd_characters,
                       format!("{} {} {:?}",
                               "Values & value labels shouldn't contain",
                               "certain 'odd' characters.",
                               &setting.setting).as_str());

        if let Some(ref mut status) = report.summary.value_odd_characters {
            if contains(&format!("{}", &value.value), &setting.setting) ||
                contains(&value.label, &setting.setting) {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  String::from(""),
                                  value.var_index,
                                  value.row);
            } else {
                status.pass += 1;
            }
        }
    }
}

/// Check for values over a specified max length
fn label_max_length(value: &Value,
                    config: &Config,
                    report: &mut Report) {
    if let Some(ref setting) = config
            .value_config
            .label_max_length {
        include_check!(report.summary.value_label_max_length,
                       format!("{} ({} characters)",
                               "Value labels cannot exceed a maximum length",
                               &setting.setting).as_str());

        if let Some(ref mut status) = report.summary.value_label_max_length {
            if value.label.len() > setting.setting as usize {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  String::from(""),
                                  value.var_index,
                                  -1);
            } else {
                status.pass += 1;
            }
        }
    }
}


/// Check for defined missing values that do not have a label
fn value_defined_missing_no_label(value: &Value,
                                  config: &Config,
                                  report: &mut Report) {
    if let Some(ref setting) = config
            .value_config
            .defined_missing_no_label {
        include_check!(report.summary.value_defined_missing_no_label,
                       "Values defined as missing must have a label");

        if let Some(ref mut status) = report.summary.value_defined_missing_no_label {
            if setting.setting &&
                value.missing == Missing::DEFINED_MISSING &&
                    value.label == "" {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  String::from(""),
                                  value.var_index,
                                  value.row);
            } else {
                status.pass += 1;
            }
        }
    }
}

