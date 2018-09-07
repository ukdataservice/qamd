use check::{contains, PostCheckFn};
use config::Config;
use readstat::context::Context;
use report::missing::Missing;
use report::{Locator, Report, Status};

use std::collections::HashSet;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec![
        primary_variable,
        system_missing_over_threshold,
        variables_with_unique_values,
        value_label_max_length,
        value_odd_characters,
    ]
}

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &Context, config: &Config, report: &mut Report) {
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
fn system_missing_over_threshold(context: &Context, config: &Config, report: &mut Report) {
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
fn variables_with_unique_values(context: &Context, config: &Config, report: &mut Report) {
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
fn value_label_max_length(context: &Context, config: &Config, report: &mut Report) {
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
                        } else {
                            status.pass += 1;
                        }
                    }
                }
            }
        }
    }
}

/// Check for odd characters in the value and value label.
/// If a value is determined to contain any odd character(s),
/// the number of fails (or warns) are incremented.
fn value_odd_characters(context: &Context, config: &Config, report: &mut Report) {
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

#[cfg(test)]
mod tests {
    use super::*;

    use check::Check;
    use config::Setting;
    use report::{Report, Value, Variable};

    use std::collections::HashMap;
    // use report::anyvalue::AnyValue;

    fn setup() -> Context {
        let mut config = Config::new();

        config.variable_config.primary_variable = Some(Setting {
            setting: String::from("foo"),
            desc: String::from("primary variable"),
        });

        let mut freq_table: HashMap<Variable, HashMap<Value, i32>> = HashMap::new();

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            temp.insert(Value::from("bar"), 3);
            temp.insert(Value::from("baz"), 2);
            temp.insert(Value::from("qux"), 5);
            freq_table.insert(Variable::from("foo"), temp.clone());

            temp.insert(Value::from("baz"), 5);
            freq_table.insert(Variable::from("bar"), temp);
        }

        Context {
            config: config,
            report: Report::new(),
            checks: Check {
                variable: vec![],
                value: vec![],
                post: vec![],
            },
            pb: None,
            variables: vec![],
            value_labels: HashMap::new(),
            frequency_table: freq_table,
        }
    }

    #[test]
    fn test_primary_variable() {
        let context = setup();
        let config = &context.config;
        let mut report = context.report.clone();

        assert!(report.metadata.case_count.is_none());

        primary_variable(&context, config, &mut report);
        if let Some(case_count) = report.metadata.case_count {
            assert_eq!(case_count, 3);
        } else {
            assert!(
                false,
                "report.metadata.case_count should be Some(i32) but is None"
            )
        }
    }

    // fn test_system_missing_over_threshold() {}
    // fn test_variables_with_unique_values() {}
    // fn test_value_label_max_length() {}
    // fn test_value_odd_characters() {}
}
