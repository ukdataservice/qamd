use config::Config;
use model::missing::Missing;
use model::value::Value;
use report::{Category, Locator, Report, Status};

use check::ValueCheckFn;

use std::collections::HashSet;

/// Register the checks with the context object
pub fn register() -> Vec<ValueCheckFn> {
    vec![value_defined_missing_no_label]
}

// Value checks

/// Check for defined missing values that do not have a label
fn value_defined_missing_no_label(value: &Value, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.metadata.value_defined_missing_no_label {
        use check::CheckName::ValueDefinedMissingNoLabel;
        include_check!(
            report.summary,
            ValueDefinedMissingNoLabel,
            &setting.desc,
            Category::Metadata
        );

        if let Some(ref mut status) = report.summary.get_mut(&ValueDefinedMissingNoLabel) {
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
    use model::variable::{Variable, VariableType};

    fn setup() -> (Value, Config, Report) {
        let value = Value {
            variable: Variable {
                index: 0,
                name: String::from("test"),
                label: String::from("test variable label"),
                type_: VariableType::Text,
                value_format: String::new(),
                value_labels: String::new(),
            },
            row: 1,
            value: AnyValue::from("foo"),
            label: "this is a value label".to_string(),
            missing: Missing::NOT_MISSING,
        };

        (value, Config::default(), Report::new())
    }

    #[test]
    fn test_value_defined_missing_no_label() {
        use check::CheckName::ValueDefinedMissingNoLabel;
        let (mut value, mut config, mut report) = setup();

        config.metadata.value_defined_missing_no_label = Some(Setting {
            setting: true,
            desc: "description from config".to_string(),
        });

        assert!(report
            .summary
            .get_mut(&ValueDefinedMissingNoLabel)
            .is_none());

        value.missing = Missing::DEFINED_MISSING;
        value.label = "".to_string();

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.get_mut(&ValueDefinedMissingNoLabel), 0, 1);

        value.missing = Missing::NOT_MISSING;

        value_defined_missing_no_label(&value, &config, &mut report);
        assert_setting!(report.summary.get_mut(&ValueDefinedMissingNoLabel), 1, 1);
    }
}
