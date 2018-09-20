use check::{ contains, PostCheckFn };
use readstat::context::Context;
use model::missing::Missing;
use report::{ Locator, Status };

use std::collections::HashSet;

use regex::Regex;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec![
        primary_variable,
        system_missing_over_threshold,
        variables_with_unique_values,
        value_label_max_length,
        value_odd_characters,
        regex_patterns,
    ]
}

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref primary_variable) = config.variable_config.primary_variable {
        if report.metadata.case_count.is_none() {
            report.metadata.case_count = Some(0);
        }

        if let Some((_variable, map)) = context
            .frequency_table
            .iter()
            .find(|(variable, _)| variable.name == primary_variable.setting)
        {
            // report count of distinct cases for this variable
            report.metadata.case_count = Some(map.keys().len() as i32);
        }
    }
}

/// Report variables with a number of system missing values over a
/// specified threhold.
fn system_missing_over_threshold(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.value_config.system_missing_value_threshold {
        include_check!(
            report.summary.system_missing_over_threshold,
            format!("{} (Threshold: {}%)", setting.desc, setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.system_missing_over_threshold {
            // map between variable and % missing

            // pull count of sysmiss values from Context.frequency_table
            // sum to percentage of sysmiss (delivered as NaN)

            for (variable, map) in &context.frequency_table {
                let sum = map.iter().fold(0, |mut sum, (_, occ)| {
                    sum += occ;
                    sum
                });

                assert_eq!(report.metadata.raw_case_count, sum);

                // compare with config threhold
                // and increment pass/fail
                if let Some((_, count)) = map.iter()
                    .find(|(value, _)| value.missing == Missing::SYSTEM_MISSING)
                {
                    let sys_miss = (*count as f32 / sum as f32) * 100.0;
                    if sys_miss > setting.setting as f32 {
                        status.fail += 1;

                        include_locators!(config, status, variable.name, variable.index, -1);
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;
        }
    }
}

/// Count the number of variables with one or more unique values
fn variables_with_unique_values(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.variable_config.variables_with_unique_values {
        include_check!(report.summary.variables_with_unique_values, &setting.desc);

        if let Some(ref mut status) = report.summary.variables_with_unique_values {
            for (variable, map) in context.frequency_table.iter() {
                if let Some(_) = map.iter().find(|(_value, occ)| *occ <= &setting.setting) {
                    status.fail += 1;

                    include_locators!(config, status, variable.name, variable.index, -1);
                } else {
                    status.pass += 1
                }
            }
        }
    }
}

/// Check for values over a specified max length
fn value_label_max_length(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.value_config.label_max_length {
        include_check!(
            report.summary.value_label_max_length,
            format!("{} ({} characters)", setting.desc, &setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.value_label_max_length {
            for variable in (*context).variables.iter() {
                if let Some(values) = (*context).frequency_table.get(&variable) {
                    for (value, _occ) in values.iter() {
                        if value.label.len() > setting.setting as usize {
                            status.fail += 1;

                            include_locators!(
                                config,
                                status,
                                value.variable.name,
                                value.variable.index,
                                -1
                            );
                        }
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;
        }
    }
}

/// Check for odd characters in the value and value label.
/// If a value is determined to contain any odd character(s),
/// the number of fails (or warns) are incremented.
fn value_odd_characters(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.value_config.odd_characters {
        include_check!(
            report.summary.value_odd_characters,
            format!("{} {:?}", setting.desc, &setting.setting).as_str()
        );

        if let Some(ref mut status) = report.summary.value_odd_characters {
            for variable in (*context).variables.iter() {
                if let Some(values) = (*context).frequency_table.get(&variable) {
                    for (value, _occ) in values.iter() {
                        if contains(&format!("{}", &value.value), &setting.setting)
                            || contains(&value.label, &setting.setting)
                        {
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
    }
}

/// Flags values that match a regex pattern
fn regex_patterns(context: &mut Context) {
    if let Some(ref setting) = context.config.value_config.regex_patterns {
        include_check!(context.report.summary.value_regex_patterns, &setting.desc);

        if let Some(ref mut status) = context.report.summary.value_regex_patterns {
            for variable in context.variables.iter() {
                for (value, _occ) in context.frequency_table.get(&variable).unwrap() {
                    for pattern in &setting.setting {
                        let re = Regex::new(&pattern).unwrap();

                        if re.is_match(&format!("{}", value.value))
                           || re.is_match(&value.label) {
                            status.fail += 1;

                            include_locators!(
                                context.config,
                                status,
                                value.variable.name,
                                value.variable.index,
                                value.row
                            );

                            break;
                        }
                    }

                }
            }

            status.pass = context.report.metadata.variable_count - status.fail;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use check::Check;
    use config::{ Config, Setting };
    use model::value::Value;
    use model::variable::Variable;
    use report::Report;

    use std::collections::HashMap;
    // use report::anyvalue::AnyValue;

    fn setup() -> Context {
        let mut freq_table: HashMap<Variable, HashMap<Value, i32>> = HashMap::new();

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();

            let mut qux = Value::from("qux");
            qux.label = String::from("this is fine");

            let mut bar = Value::from("bar#");
            bar.label = String::from("this is far too long to pass the test");

            temp.insert(bar, 3);
            temp.insert(Value::from("!baz"), 3);
            temp.insert(qux, 4);

            freq_table.insert(Variable::from("first"), temp.clone());
        }

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            let mut missing_value: Value = Value::from("");
            missing_value.missing = Missing::SYSTEM_MISSING;

            temp.insert(Value::from("g@regs"), 2);
            temp.insert(missing_value, 8);

            freq_table.insert(Variable::from("second"), temp);
        }

        let mut report = Report::new();
        report.metadata.variable_count = 2;
        report.metadata.raw_case_count = 10;

        Context {
            config: Config::new(),
            report: report,
            checks: Check {
                variable: vec![],
                value: vec![],
                post: vec![],
            },
            pb: None,
            variables: vec![
                Variable::from("first"),
                Variable::from("second")
            ],
            value_labels: HashMap::new(),
            frequency_table: freq_table,
        }
    }

    #[test]
    fn test_primary_variable() {
        let mut context = setup();
        assert!(context.report.metadata.case_count.is_none());

        context.config.variable_config.primary_variable = Some(Setting {
            setting: String::from("first"),
            desc: String::from("primary variable"),
        });

        primary_variable(&mut context);
        if let Some(case_count) = context.report.metadata.case_count {
            assert_eq!(case_count, 3);
        } else {
            assert!(
                false,
                "report.metadata.case_count should be Some(i32) but is None"
            )
        }
    }

    #[test]
    fn test_system_missing_over_threshold() {
        let mut context = setup();
        assert!(context.report.summary.system_missing_over_threshold.is_none());

        context.config.value_config.system_missing_value_threshold = Some(Setting {
            setting: 25,
            desc: String::from("sysmiss values over a threshold")
        });

        system_missing_over_threshold(&mut context);
        assert_setting!(context.report.summary.system_missing_over_threshold, 1, 1);
    }

    #[test]
    fn test_variables_with_unique_values() {
        let mut context = setup();
        assert!(context.report.summary.variables_with_unique_values.is_none());

        context.config.variable_config.variables_with_unique_values = Some(Setting {
            setting: 2,
            desc: String::from("outliers as defined by the threshold"),
        });

        variables_with_unique_values(&mut context);
        assert_setting!(context.report.summary.variables_with_unique_values, 1, 1);
    }

    #[test]
    fn test_value_label_max_length() {
        let mut context = setup();
        assert!(context.report.summary.value_label_max_length.is_none());

        context.config.value_config.label_max_length = Some(Setting {
            setting: 20,
            desc: String::from("value labels cannot be too long"),
        });

        value_label_max_length(&mut context);
        assert_setting!(context.report.summary.value_label_max_length, 1, 1);
    }

    #[test]
    fn test_value_odd_characters() {
        let mut context = setup();
        assert!(context.report.summary.value_odd_characters.is_none());

        context.config.value_config.odd_characters = Some(Setting {
            setting: vec!["#", "@", "!"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            desc: String::from("value names and labels shouldn't contain some characters"),
        });

        value_odd_characters(&mut context);
        assert_setting!(context.report.summary.value_odd_characters, 2, 3);
    }

    #[test]
    fn test_regex_patterns() {
        let mut context = setup();
        assert!(context.report.summary.value_regex_patterns.is_none());

        context.config.value_config.regex_patterns = Some(Setting {
            setting: vec![r"^qux".to_string()],
            desc: "description from config".to_string(),
        });

        regex_patterns(&mut context);
        assert_setting!(context.report.summary.value_regex_patterns, 1, 1);
    }
}
