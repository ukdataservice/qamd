use config::Config;
use report::missing::Missing;
use report::{Locator, Report, Status, Value};

use check::ValueCheckFn;

use std::collections::HashSet;

use regex::Regex;

/// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec![value_defined_missing_no_label, regex_patterns]
}

// Value checks

/// Check for defined missing values that do not have a label
fn value_defined_missing_no_label(value: &Value, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.value_config.defined_missing_no_label {
        include_check!(report.summary.value_defined_missing_no_label, &setting.desc);

        if let Some(ref mut status) = report.summary.value_defined_missing_no_label {
            if setting.setting && value.missing == Missing::DEFINED_MISSING && value.label == "" {
                status.fail += 1;

                include_locators!(
                    config,
                    status,
                    value.variable.name,
                    value.variable.index,
                    value.row
                );
            } else {
                status.pass += 1;
            }
        }
    }
}

/// Flags values that match a regex pattern
fn regex_patterns(value: &Value, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.value_config.regex_patterns {
        include_check!(report.summary.value_regex_patterns, &setting.desc);

        if let Some(ref mut status) = report.summary.value_regex_patterns {
            for pattern in &setting.setting {
                let re = Regex::new(&pattern).unwrap();

                if re.is_match(&format!("{}", value.value)) || re.is_match(&value.label) {
                    status.fail += 1;

                    include_locators!(
                        config,
                        status,
                        value.variable.name,
                        value.variable.index,
                        value.row
                    );
                } else {
                    status.pass += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use config::Setting;
    use report::anyvalue::AnyValue;
    use report::Variable;

    fn setup() -> (Value, Config, Report) {
        let value = Value {
            variable: Variable {
                index: 0,
                name: String::from("test"),
                label: String::from("test variable label"),
                value_format: String::new(),
                value_labels: String::new(),
            },
            row: 1,
            value: AnyValue::from("foo"),
            label: "this is a value label".to_string(),
            missing: Missing::NOT_MISSING,
        };

        let mut config = Config::new();
        config.value_config.regex_patterns = Some(Setting {
            setting: vec![r"^foo".to_string()],
            desc: "description from config".to_string(),
        });

        config.value_config.defined_missing_no_label = Some(Setting {
            setting: true,
            desc: "description from config".to_string(),
        });

        (value, config, Report::new())
    }

    #[test]
    fn test_value_defined_missing_no_label() {
        let (mut value, config, mut report) = setup();
        assert!(report.summary.value_defined_missing_no_label.is_none());

        value.missing = Missing::DEFINED_MISSING;
        value.label = "".to_string();

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.value_defined_missing_no_label, 0, 1);

        value.missing = Missing::NOT_MISSING;

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.value_defined_missing_no_label, 1, 1);
    }

    #[test]
    fn test_regex_patterns() {
        let (mut value, config, mut report) = setup();
        assert!(report.summary.value_regex_patterns.is_none());

        regex_patterns(&value, &config, &mut report);
        assert_setting!(report.summary.value_regex_patterns, 0, 1);

        // value won't match regex
        value.value = AnyValue::from("bar");

        regex_patterns(&value, &config, &mut report);
        assert_setting!(report.summary.value_regex_patterns, 1, 1);
    }
}
