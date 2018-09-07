use check::{contains, VariableCheckFn};
use config::Config;
use report::{Locator, Report, Status, Variable};

use std::collections::HashSet;

// Register the checks
pub fn register() -> Vec<VariableCheckFn> {
    vec![
        missing_variable_labels,
        date_format,
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
        include_check!(report.summary.date_format, &setting.desc);
        let date_time_specifiers = &setting.setting;

        if let Some(ref mut status) = report.summary.date_format {
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
        include_check!(report.summary.missing_variable_labels, &setting.desc);

        if setting.setting {
            if let Some(ref mut status) = report.summary.missing_variable_labels {
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
        include_check!(
            report.summary.variable_label_max_length,
            format!("{} ({} characters)", setting.desc, &setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.variable_label_max_length {
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
        include_check!(
            report.summary.variable_odd_characters,
            format!("{} {:?}", setting.desc, setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.variable_odd_characters {
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
    fn test_missing_variable_labels() {
        let (mut variable, config, mut report) = setup();
        assert!(report.summary.missing_variable_labels.is_none());

        missing_variable_labels(&variable, &config, &mut report);
        assert_setting!(report.summary.missing_variable_labels, 0, 1);

        variable.label = String::from("variable label");
        missing_variable_labels(&variable, &config, &mut report);
        assert_setting!(report.summary.missing_variable_labels, 1, 1);
    }

    #[test]
    fn test_date_format() {
        let (mut variable, config, mut report) = setup();
        assert!(report.summary.date_format.is_none());

        date_format(&variable, &config, &mut report);
        assert_setting!(report.summary.date_format, 1, 0);

        variable.value_format = String::from("SDATE");
        date_format(&variable, &config, &mut report);
        assert_setting!(report.summary.date_format, 1, 1);
    }

    #[test]
    fn test_variable_label_max_length() {
        let (mut variable, config, mut report) = setup();
        assert!(report.summary.variable_label_max_length.is_none());

        variable.label = String::from("variable label");
        variable_label_max_length(&variable, &config, &mut report);
        assert_setting!(report.summary.variable_label_max_length, 1, 0);

        variable.label = String::from("variable label is far too long to pass the test");
        variable_label_max_length(&variable, &config, &mut report);
        assert_setting!(report.summary.variable_label_max_length, 1, 1);
    }

    #[test]
    fn test_variable_odd_characters() {
        let (mut variable, config, mut report) = setup();
        assert!(report.summary.variable_odd_characters.is_none());

        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.variable_odd_characters, 1, 0);

        variable.name = String::from("foo@");
        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.variable_odd_characters, 1, 1);

        variable.name = String::from("foo");
        variable.label = String::from("bad #label");
        variable_odd_characters(&variable, &config, &mut report);
        assert_setting!(report.summary.variable_odd_characters, 1, 2);
    }
}
