use check::{contains, only_contains, read_file, PostCheckFn};
use model::missing::Missing;
use model::value::Value;
use model::variable::Variable;
use readstat::context::Context;
use report::{Locator, Status};

use std::collections::HashMap;
use std::collections::HashSet;

use regex::Regex;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec![
        primary_variable,
        duplicate_values,
        system_missing_over_threshold,
        variables_with_unique_values,
        value_label_max_length,
        value_odd_characters,
        regex_patterns,
        spellcheck,
        bad_filename,
    ]
}

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref variable_config) = config.variable_config {
        if let Some(ref primary_variable) = variable_config.primary_variable {
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
}

/// Notify if a variable has duplicate values, and where they are
fn duplicate_values(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref value_config) = config.value_config {
        if let Some(ref setting) = value_config.duplicate_values {
            use check::CheckName::DuplicateValues;
            include_check!(
                report.summary,
                DuplicateValues,
                format!("{} (On variables {:?})", setting.desc, setting.setting).as_str()
            );

            if let Some(ref mut status) = report.summary.get_mut(&DuplicateValues) {
                let case_count = &report.metadata.raw_case_count;

                context
                    .frequency_table
                    .iter()
                    .filter(move |(variable, _)| setting.setting.contains(&variable.name))
                    .for_each(|(variable, map)| {
                        let count = map.values().filter(|occ| **occ == 1).count() as i32;
                        if count != *case_count {
                            status.fail += 1;

                            include_locators!(config, status, variable.name, variable.index, -1);
                        }
                    });

                status.pass = setting.setting.len() as i32 - status.fail;
            }
        }
    }
}

/// Report variables with a number of system missing values over a
/// specified threhold.
fn system_missing_over_threshold(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref value_config) = config.value_config {
        if let Some(ref setting) = value_config.system_missing_value_threshold {
            use check::CheckName::SystemMissingOverThreshold;
            include_check!(
                report.summary,
                SystemMissingOverThreshold,
                format!("{} (Threshold: {}%)", setting.desc, setting.setting).as_str()
            );

            if let Some(ref mut status) = report.summary.get_mut(&SystemMissingOverThreshold) {
                // map between variable and % missing

                // pull count of sysmiss values from Context.frequency_table
                // sum to percentage of sysmiss (delivered as NaN)

                for (variable, map) in &context.frequency_table {
                    let sum = map.iter().fold(0, |mut sum, (_, occ)| {
                        sum += occ;
                        sum
                    });

                    assert_eq!(
                        report.metadata.raw_case_count, sum,
                        "case_count {} does not align with sum {} for variable {}",
                        report.metadata.raw_case_count, sum, variable.name
                    );

                    // compare with config threhold
                    // and increment pass/fail
                    if let Some((_, count)) = map
                        .iter()
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
}

/// Count the number of variables with one or more unique values
fn variables_with_unique_values(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref variable_config) = config.variable_config {
        if let Some(ref setting) = variable_config.variables_with_unique_values {
            use check::CheckName::VariablesWithUniqueValues;
            include_check!(report.summary, VariablesWithUniqueValues, &setting.desc);

            if let Some(ref mut status) = report.summary.get_mut(&VariablesWithUniqueValues) {
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
}

/// Check for values over a specified max length
fn value_label_max_length(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref value_config) = config.value_config {
        if let Some(ref setting) = value_config.label_max_length {
            use check::CheckName::ValueLabelMaxLength;
            include_check!(
                report.summary,
                ValueLabelMaxLength,
                format!("{} ({} characters)", setting.desc, &setting.setting).as_str()
            );

            if let Some(ref mut status) = report.summary.get_mut(&ValueLabelMaxLength) {
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
}

/// Check for odd characters in the value and value label.
/// If a value is determined to contain any odd character(s),
/// the number of fails (or warns) are incremented.
fn value_odd_characters(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref value_config) = config.value_config {
        if let Some(ref setting) = value_config.odd_characters {
            use check::CheckName::ValueOddCharacters;
            include_check!(
                report.summary,
                ValueOddCharacters,
                format!("{} {:?}", setting.desc, &setting.setting).as_str()
            );

            if let Some(ref mut status) = report.summary.get_mut(&ValueOddCharacters) {
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
}

/// Flags values that match a regex pattern
fn regex_patterns(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref value_config) = config.value_config {
        if let Some(ref setting) = value_config.regex_patterns {
            use check::CheckName::ValueRegexPatterns;
            include_check!(report.summary, ValueRegexPatterns, &setting.desc);

            if let Some(ref mut status) = report.summary.get_mut(&ValueRegexPatterns) {
                for variable in context.variables.iter() {
                    for (value, _occ) in context.frequency_table.get(&variable).unwrap() {
                        for pattern in &setting.setting {
                            let re = Regex::new(&pattern).unwrap();

                            if re.is_match(&format!("{}", value.value)) || re.is_match(&value.label)
                            {
                                status.fail += 1;

                                include_locators!(
                                    config,
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

                status.pass = report.metadata.variable_count - status.fail;
            }
        }
    }
}

fn spellcheck(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if config.spellcheck.is_none() {
        return;
    }

    let setting_desc = match config.spellcheck {
        Some(ref setting) => &setting.desc,
        None => "",
    };

    let dictonaries_paths = config.get_dictionaries();

    let words: Vec<String> = dictonaries_paths
        .iter()
        .map(|path| read_file(path))
        .filter_map(|result| result.ok())
        .collect();

    use check::CheckName::Spellcheck;
    include_check!(report.summary, Spellcheck, &setting_desc);

    if let Some(ref mut status) = report.summary.get_mut(&Spellcheck) {
        for variable in context.variables.iter() {
            if only_contains(&variable.label, &words) {
                include_locators!(config, status, variable.name, variable.index, -1);
                status.fail += 1;
            }

            for (value, _occ) in context.frequency_table.get(&variable).unwrap() {
                if !only_contains(&value.label, &words) {
                    include_locators!(
                        config,
                        status,
                        value.variable.name,
                        value.variable.index,
                        value.row
                    );
                    status.fail += 1;
                }
            }
        }

        status.pass = total_checked(&context.frequency_table) - status.fail;
    }
}

fn total_checked(frequency_table: &HashMap<Variable, HashMap<Value, i32>>) -> i32 {
    frequency_table
        .iter()
        .fold(0, |total, (_var, val)| total + val.len() as i32 + 1)
}

fn bad_filename(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref bad_filename) = config.bad_filename {
        let pattern = &bad_filename.setting;
        let file_name = &report.metadata.file_name;
        let re = Regex::new(pattern).unwrap();

        use check::CheckName::BadFileName;
        let mut status = Status::new(&bad_filename.desc);
        let mut locators: HashSet<Locator> = HashSet::new();

        if !re.is_match(file_name) {
            status.fail += 1;

            locators.insert(Locator::new("".to_string(),
                                         -1, -1,
                                         Some(format!("file name: {} {} {}",
                                                 file_name,
                                                 "did not match the given pattern:",
                                                 pattern))));
            status.locators = Some(locators);
        } else {
            status.pass += 1;
        }

        report.summary.insert(BadFileName, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use check::Check;
    use config::{Config, Setting};
    use model::anyvalue::AnyValue;
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
            bar.label = String::from("this is far too long to pss the test");

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

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            let variable = Variable::from("badid");

            for i in 1i32..=10 {
                if i == 4 {
                    continue;
                }
                let quant = if i == 1 { 2 } else { 1 };

                temp.insert(
                    Value {
                        variable: variable.clone(),
                        row: i,
                        value: AnyValue::from(i),
                        label: String::new(),
                        missing: Missing::NOT_MISSING,
                    },
                    quant,
                );
            }

            freq_table.insert(variable, temp);
        }

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            let variable = Variable::from("id");
            for i in 1i32..=10 {
                temp.insert(
                    Value {
                        variable: variable.clone(),
                        row: i,
                        value: AnyValue::from(i),
                        label: String::new(),
                        missing: Missing::NOT_MISSING,
                    },
                    1,
                );
            }

            freq_table.insert(variable, temp);
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
            variables: vec![Variable::from("first"), Variable::from("second")],
            value_labels: HashMap::new(),
            frequency_table: freq_table,
        }
    }

    #[test]
    fn test_primary_variable() {
        let mut context = setup();
        assert!(context.report.metadata.case_count.is_none());

        if let Some(ref mut variable_config) = context.config.variable_config {
            variable_config.primary_variable = Some(Setting {
                setting: String::from("first"),
                desc: String::from("primary variable"),
            });
        }

        primary_variable(&mut context);
        if let Some(case_count) = context.report.metadata.case_count {
            assert_eq!(case_count, 3);
        } else {
            assert!(
                false,
                concat!(
                    "report.metadata.case_count should ",
                    "be Some(i32) but is None"
                )
            );
        }
    }

    #[test]
    fn test_duplicate_values() {
        let mut context = setup();

        use check::CheckName::DuplicateValues;

        assert!(context.report.summary.get_mut(&DuplicateValues).is_none());

        if let Some(ref mut value_config) = context.config.value_config {
            value_config.duplicate_values = Some(Setting {
                setting: vec!["id", "badid"].iter().map(|s| s.to_string()).collect(),
                desc: "description from config".to_string(),
            });
        }

        duplicate_values(&mut context);
        assert_setting!(context.report.summary.get(&DuplicateValues), 1, 1);
    }

    #[test]
    fn test_system_missing_over_threshold() {
        let mut context = setup();

        use check::CheckName::SystemMissingOverThreshold;

        assert!(context
            .report
            .summary
            .get(&SystemMissingOverThreshold)
            .is_none());

        if let Some(ref mut value_config) = context.config.value_config {
            value_config.system_missing_value_threshold = Some(Setting {
                setting: 25,
                desc: String::from("sysmiss values over a threshold"),
            });
        }

        system_missing_over_threshold(&mut context);
        assert_setting!(
            context.report.summary.get(&SystemMissingOverThreshold),
            1,
            1
        );
    }

    #[test]
    fn test_variables_with_unique_values() {
        let mut context = setup();

        use check::CheckName::VariablesWithUniqueValues;

        assert!(context
            .report
            .summary
            .get(&VariablesWithUniqueValues)
            .is_none());

        if let Some(ref mut variable_config) = context.config.variable_config {
            variable_config.variables_with_unique_values = Some(Setting {
                setting: 2,
                desc: String::from("outliers as defined by the threshold"),
            });
        }

        variables_with_unique_values(&mut context);
        assert_setting!(context.report.summary.get(&VariablesWithUniqueValues), 1, 3);
    }

    #[test]
    fn test_value_label_max_length() {
        let mut context = setup();

        use check::CheckName::ValueLabelMaxLength;

        assert!(context.report.summary.get(&ValueLabelMaxLength).is_none());

        if let Some(ref mut value_config) = context.config.value_config {
            value_config.label_max_length = Some(Setting {
                setting: 20,
                desc: String::from("value labels cannot be too long"),
            });
        }

        value_label_max_length(&mut context);
        assert_setting!(context.report.summary.get(&ValueLabelMaxLength), 1, 1);
    }

    #[test]
    fn test_value_odd_characters() {
        let mut context = setup();

        use check::CheckName::ValueOddCharacters;

        assert!(context.report.summary.get(&ValueOddCharacters).is_none());

        if let Some(ref mut value_config) = context.config.value_config {
            value_config.odd_characters = Some(Setting {
                setting: vec!["#", "@", "!"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
                desc: String::from("value names and labels shouldn't contain some characters"),
            });
        }

        value_odd_characters(&mut context);
        assert_setting!(context.report.summary.get(&ValueOddCharacters), 2, 3);
    }

    #[test]
    fn test_regex_patterns() {
        let mut context = setup();

        use check::CheckName::ValueRegexPatterns;

        assert!(context.report.summary.get(&ValueRegexPatterns).is_none());

        if let Some(ref mut value_config) = context.config.value_config {
            value_config.regex_patterns = Some(Setting {
                setting: vec![r"^qux".to_string()],
                desc: "description from config".to_string(),
            });
        }

        regex_patterns(&mut context);
        assert_setting!(context.report.summary.get(&ValueRegexPatterns), 1, 1);
    }

    #[test]
    fn test_spellcheck() {
        let mut context = setup();

        use check::CheckName::Spellcheck;

        assert!(context.report.summary.get(&Spellcheck).is_none());

        context.config.spellcheck = Some(Setting {
            setting: vec!["test/words.txt".to_string()],
            desc: "spellcheck: description from config".to_string(),
        });

        spellcheck(&mut context);
        assert_setting!(context.report.summary.get(&Spellcheck), 23, 5);
    }

    #[test]
    fn test_bad_filename() {
        let mut context = setup();

        use check::CheckName::BadFileName;

        assert!(context.report.summary.get(&BadFileName).is_none());

        context.config.bad_filename = Some(Setting {
            setting: "^([a-zA-Z0-9]+)\\.([a-zA-Z0-9]+)$".to_string(),
            desc: "filename must match pattern".to_string(),
        });

        context.report.metadata.file_name = "goodfilename.dta".to_string();
        bad_filename(&mut context);
        assert_setting!(context.report.summary.get(&BadFileName), 1, 0);

        context.report.metadata.file_name = "bad& filename.foo".to_string();
        bad_filename(&mut context);
        assert_setting!(context.report.summary.get(&BadFileName), 0, 1);
    }
}
