use config::Config;
use model::missing::Missing;
use model::value::Value;
use report::{ Locator, Report, Status };

use check::ValueCheckFn;

use std::collections::HashSet;

/// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec![value_defined_missing_no_label]
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

#[cfg(test)]
mod tests {
    use super::*;

    use config::Setting;
    use model::anyvalue::AnyValue;
    use model::variable::Variable;

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

        (value, Config::new(), Report::new())
    }

    #[test]
    fn test_value_defined_missing_no_label() {
        let (mut value, mut config, mut report) = setup();
        assert!(report.summary.value_defined_missing_no_label.is_none());

        config.value_config.defined_missing_no_label = Some(Setting {
            setting: true,
            desc: "description from config".to_string(),
        });

        value.missing = Missing::DEFINED_MISSING;
        value.label = "".to_string();

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.value_defined_missing_no_label, 0, 1);

        value.missing = Missing::NOT_MISSING;

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.value_defined_missing_no_label, 1, 1);
    }

}
