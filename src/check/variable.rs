use check::{contains, VariableCheckFn};
use config::Config;
use model::variable::Variable;
use report::{Locator, Report, Status};

use std::collections::HashSet;

// Register the checks
pub fn register() -> Vec<VariableCheckFn> {
    vec![
        date_format,
        missing_variable_labels,
        variable_label_max_length,
        variable_odd_characters,
    ]
}

/// Variable checks

fn date_format(variable: &Variable, config: &Config, report: &mut Report) {
    // refer here for the docs on the date format. ReadStat internally
    // attempts to treat data as-if it were just Stata.
    // https://www.stata.com/help.cgi?datetime_display_formats

    if let Some(ref setting) = config.variable_config.date_format {
        use check::CheckName::DateFormat;
        include_check!(report.summary, DateFormat, &setting.desc);

        let date_time_specifiers = &setting.setting;

        if let Some(ref mut status) = report.summary.get_mut(&DateFormat) {
            if contains(&variable.value_format, &date_time_specifiers) {
                status.fail += 1;

                include_locators!(config, status, variable.name, variable.index, -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

fn missing_variable_labels(variable: &Variable, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.variable_config.missing_variable_labels {
        use check::CheckName::MissingVariableLabels;
        include_check!(report.summary, MissingVariableLabels, &setting.desc);

        if setting.setting {
            if let Some(ref mut status) = report.summary.get_mut(&MissingVariableLabels) {
                if variable.label.is_empty() {
                    status.fail += 1;

                    include_locators!(config, status, variable.name, variable.index, -1);
                } else {
                    status.pass += 1;
                }
            }
        }
    }
}

fn variable_label_max_length(variable: &Variable, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.variable_config.label_max_length {
        use check::CheckName::VariableLabelMaxLength;
        include_check!(
            report.summary,
            VariableLabelMaxLength,
            format!("{} ({} characters)", setting.desc, &setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.get_mut(&VariableLabelMaxLength) {
            if variable.label.len() > setting.setting as usize {
                status.fail += 1;

                include_locators!(config, status, variable.name, variable.index, -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

fn variable_odd_characters(variable: &Variable, config: &Config, report: &mut Report) {
    if let Some(ref setting) = config.variable_config.odd_characters {
        use check::CheckName::VariableOddCharacters;
        include_check!(
            report.summary,
            VariableOddCharacters,
            format!("{} {:?}", setting.desc, setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.get_mut(&VariableOddCharacters) {
            if contains(&variable.name, &setting.setting)
                || contains(&variable.label, &setting.setting)
            {
                status.fail += 1;

                include_locators!(config, status, variable.name, variable.index, -1);
            } else {
                status.pass += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use config::{Config, Setting};
    use report::Report;

    fn setup() -> (Variable, Config, Report) {
        let variable = Variable::from("foo");

        let mut config = Config::new();
        config.variable_config.missing_variable_labels = Some(Setting {
            setting: true,
            desc: String::from("variables with no labels"),
        });

        config.variable_config.date_format = Some(Setting {
            setting: vec!["SDATE", "TIME", "JJJ"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            desc: String::from("date format"),
        });

        config.variable_config.label_max_length = Some(Setting {
            setting: 15,
            desc: String::from("label max length"),
        });

        config.variable_config.odd_characters = Some(Setting {
            setting: vec!["#", "@"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            desc: String::from("variable odd characters"),
        });

        (variable, config, Report::new())
    }

    #[test]
    fn test_date_format() {
        let (mut variable, config, mut report) = setup();
        use check::CheckName::DateFormat;

        assert!(report.summary.get(&DateFormat).is_none());

        date_format(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&DateFormat), 1, 0);

        variable.value_format = String::from("SDATE");
        date_format(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&DateFormat), 1, 1);
    }

    #[test]
    fn test_missing_variable_labels() {
        let (mut variable, config, mut report) = setup();
        use check::CheckName::MissingVariableLabels;

        assert!(report.summary.get(&MissingVariableLabels).is_none());

        missing_variable_labels(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&MissingVariableLabels), 0, 1);

        variable.label = String::from("variable label");
        missing_variable_labels(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&MissingVariableLabels), 1, 1);
    }

    #[test]
    fn test_variable_label_max_length() {
        let (mut variable, config, mut report) = setup();
        use check::CheckName::VariableLabelMaxLength;

        assert!(report.summary.get(&VariableLabelMaxLength).is_none());

        variable.label = String::from("variable label");
        variable_label_max_length(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&VariableLabelMaxLength), 1, 0);

        variable.label = String::from("variable label is far too long to pass the test");
        variable_label_max_length(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&VariableLabelMaxLength), 1, 1);
    }

    #[test]
    fn test_variable_odd_characters() {
        let (mut variable, config, mut report) = setup();
        use check::CheckName::VariableOddCharacters;

        assert!(report.summary.get(&VariableOddCharacters).is_none());

        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&VariableOddCharacters), 1, 0);

        variable.name = String::from("foo@");
        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&VariableOddCharacters), 1, 1);

        variable.name = String::from("foo");
        variable.label = String::from("bad #label");
        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.get(&VariableOddCharacters), 1, 2);
    }
}
