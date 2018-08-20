
use config::Config;
use report::{ Report, Value, Status, Locator };
use report::missing::Missing;

use check::ValueCheckFn;

use std::collections::HashSet;

/// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec!(value_defined_missing_no_label)
}

// Value checks

/// Check for defined missing values that do not have a label
fn value_defined_missing_no_label(value: &Value,
                                  config: &Config,
                                  report: &mut Report) {
    if let Some(ref setting) = config
            .value_config
            .defined_missing_no_label {
        include_check!(report.summary.value_defined_missing_no_label,
                       &setting.desc);

        if let Some(ref mut status) = report.summary.value_defined_missing_no_label {
            if setting.setting &&
                value.missing == Missing::DEFINED_MISSING &&
                    value.label == "" {
                status.fail += 1;

                include_locators!(config,
                                  status,
                                  value.variable.name,
                                  value.variable.index,
                                  value.row);
            } else {
                status.pass += 1;
            }
        }
    }
}

