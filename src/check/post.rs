
use readstat::context::Context;
use config::Config;
use report::{ Report, Status, Locator };
use report::missing::Missing;
use check::{ PostCheckFn, contains };

use std::collections::HashSet;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec!(primary_variable,
         system_missing_over_threshold,
         variables_with_unique_values,
         value_label_max_length,
         value_odd_characters)
}

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &Context,
                    config: &Config,
                    report: &mut Report) {
    if let Some(ref primary_variable) = config
        .variable_config
        .primary_variable {

        if report.metadata.case_count.is_none() {
            report.metadata.case_count = Some(0);
        }

        if let Some((_variable, map)) = context.frequency_table
            .iter().find(|(variable, _)| {
            variable.name == primary_variable.setting
        }) {
            // report count of distinct cases for this variable
            report.metadata.case_count = Some(map.keys().len() as i32);
        }
    }
}

/// Report variables with a number of system missing values over a
/// specified threhold.
fn system_missing_over_threshold(context: &Context,
                                 config: &Config,
                                 report: &mut Report) {
    if let Some(ref setting) = config
            .value_config
            .system_missing_value_threshold {
        include_check!(report.summary.system_missing_over_threshold,
                       format!("{} (Threshold: {}%)",
                               setting.desc,
                               setting.setting).as_str());

        if let Some(ref mut status) = report
                .summary
                .system_missing_over_threshold {
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
                if let Some((_, count)) = map
                    .iter().find(|(value, _)| {
                        value.missing == Missing::SYSTEM_MISSING
                    }) {
                    let sys_miss = (*count as f32 / sum as f32) * 100.0;
                    if sys_miss > setting.setting as f32 {
                        status.fail += 1;

                        include_locators!(config,
                                          status,
                                          variable.name,
                                          variable.index,
                                          -1);
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;

        }
    }
}

/// Count the number of variables with one or more unique values
fn variables_with_unique_values(context: &Context,
                                config: &Config,
                                report: &mut Report) {
    if let Some(ref setting) = config
        .variable_config
        .variables_with_unique_values {
        include_check!(report.summary.variables_with_unique_values,
                       &setting.desc);

        if let Some(ref mut status) = report.summary
            .variables_with_unique_values {
            for (variable, map) in context.frequency_table.iter() {
                if let Some(_) = map.iter().find(|(_value, occ)| {
                    *occ <= &setting.setting
                }) {
                    status.fail += 1;

                    include_locators!(config,
                                      status,
                                      variable.name,
                                      variable.index,
                                      -1);
                } else {
                    status.pass += 1
                }
            }
        }
    }
}

/// Check for values over a specified max length
fn value_label_max_length(context: &Context,
                          config: &Config,
                          report: &mut Report) {
    if let Some(ref setting) = config.value_config.label_max_length {
        include_check!(report.summary.value_label_max_length,
                       format!("{} ({} characters)",
                               setting.desc,
                               &setting.setting).as_str());

        if let Some(ref mut status) = report.summary.value_label_max_length {
            for variable in (*context).variables.iter() {
                if let Some(values) = (*context).frequency_table.get(&variable) {
                    for (value, _occ) in values.iter() {
                        if value.label.len() > setting.setting as usize {
                            status.fail += 1;

                            include_locators!(config,
                                              status,
                                              value.variable.name,
                                              value.variable.index,
                                              -1);
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
fn value_odd_characters(context: &Context,
                        config: &Config,
                        report: &mut Report) {
    if let Some(ref setting) = config.value_config.odd_characters {
        include_check!(report.summary.value_odd_characters,
                       format!("{} {:?}",
                               setting.desc,
                               &setting.setting).as_str());

        if let Some(ref mut status) = report.summary.value_odd_characters {
            for variable in (*context).variables.iter() {
                if let Some(values) = (*context).frequency_table.get(&variable) {
                    for (value, _occ) in values.iter() {
                        if contains(&format!("{}", &value.value), &setting.setting) ||
                            contains(&value.label, &setting.setting) {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Setting;
    use check::Check;
    use std::collections::HashMap;
    // use report::anyvalue::AnyValue;

    fn setup() -> Context {
        let mut config = Config::new();
        config.value_config.regex_patterns = Some(Setting {
            setting: vec!(r"^foo".to_string()),
            desc: "description from config".to_string(),
        });

        config.value_config.defined_missing_no_label = Some(Setting {
            setting: true,
            desc: "description from config".to_string(),
        });

        Context {
            config: config,
            report: Report::new(),
            checks: Check::new(),
            pb: None,
            variables: vec!(),
            value_labels: HashMap::new(),
            frequency_table: HashMap::new(),
        }
    }

    // #[test]
    // fn test_name() {
    // }
}


