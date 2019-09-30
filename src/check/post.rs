use check::{contains, PostCheckFn};
use check::dictionary::{dictionary, spellcheck_predicate, stopword_predicate};
use model::variable::{Variable, VariableType};
use model::anyvalue::AnyValue;
use model::missing::Missing;
use readstat::context::Context;
use report::{Category, Locator, Status};

use std::collections::{HashSet, HashMap};

use regex::Regex;

/// Returns a vec of the functions provided by this module
pub fn register() -> Vec<PostCheckFn> {
    vec![
        // Basic File Checks
        bad_filename,

        // Metadata
        primary_variable,

        value_label_odd_characters,
        value_label_max_length,
        value_label_spellcheck,

        variable_label_spellcheck,

        // Data Integrity
        duplicate_values,
        string_value_odd_characters,
        system_missing_over_threshold,
        string_value_spellcheck,

        //  Disclosure Risk
        regex_patterns,
        unique_values,
        string_value_stopword,
    ]
}

// Basic Flle Checks

/// Filename must fit the provided regex pattern
fn bad_filename(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref bad_filename) = config.basic_file_checks.bad_filename {
        let pattern = &bad_filename.setting;
        let file_name = &report.metadata.file_name;
        let re = Regex::new(pattern).unwrap();

        use check::CheckName::BadFileName;
        let mut status = Status::new(&bad_filename.desc, Category::BasicFile);
        let mut locators: HashSet<Locator> = HashSet::new();

        if !re.is_match(file_name) {
            status.fail += 1;

            locators.insert(Locator::new("".to_string(), -1, -1));
            status.locators = Some(locators);
        } else {
            status.pass += 1;
        }

        report.summary.insert(BadFileName, status);
    }
}

// Metadata

/// Count the number of cases using the provided primary variable_count
fn primary_variable(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref primary_variable) = config.metadata.primary_variable {
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

/// Check for odd characters in the value label.
/// If a value is determined to contain any odd character(s),
/// the number of fails (or warns) are incremented.
/// Number represents the quantity of value labels that failed.
fn value_label_odd_characters(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.metadata.value_label_odd_characters {
        use check::CheckName::ValueLabelOddCharacters;
        include_check!(
            report.summary,
            ValueLabelOddCharacters,
            format!("{} {:?}", setting.desc, &setting.setting).as_str(),
            Category::Metadata
        );

        if let Some(ref mut status) = report.summary.get_mut(&ValueLabelOddCharacters) {
            for variable in (*context).variables.iter() {
                if let Some(value_labels) = (*context).value_labels.get(&variable.value_labels) {
                    for (_value, label) in value_labels.iter() {
                        if contains(label, &setting.setting) {
                            status.fail += 1;

                            include_locators!(
                                config,
                                status,
                                variable.name,
                                variable.index,
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

/// Check for values over a specified max length
fn value_label_max_length(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.metadata.value_label_max_length {
        use check::CheckName::ValueLabelMaxLength;
        include_check!(
            report.summary,
            ValueLabelMaxLength,
            format!("{} ({} characters)", setting.desc, &setting.setting).as_str(),
            Category::Metadata
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

/// Spellcheck value labels
fn value_label_spellcheck(context: &mut Context) {
    use check::CheckName::ValueLabelSpellcheck;

    let mut mapping: HashMap<String, &Variable> = HashMap::new();

    for variable in context.variables.iter()
        .filter(|v| !v.value_labels.is_empty()) {

        if let Some(labels) = context.value_labels.get(&variable.value_labels) {
            for (_value, label) in labels.iter() {
                mapping.insert(label.clone(), variable);
            }
        }
    }

    let words: HashMap<String, Locator> = mapping.iter()
        .map(|(k, v)| (k.clone(), Locator::from(v.clone())))
        .collect();

    dictionary(context, ValueLabelSpellcheck, &words, spellcheck_predicate);
}

/// Spellcheck variable labels
fn variable_label_spellcheck(context: &mut Context) {
    use check::CheckName::VariableLabelSpellcheck;

    let mut words: HashMap<String, Locator> = HashMap::new();

    for variable in context.variables.iter() {
        words.insert(variable.label.clone(), Locator::from(variable));
    }

    dictionary(context, VariableLabelSpellcheck, &words, spellcheck_predicate);
}

/// Spellcheck string values
fn string_value_spellcheck(context: &mut Context) {
    use check::CheckName::StringValueSpellcheck;

    let variables: Vec<Variable> = context.variables.iter()
        .filter(|v| v.type_ == VariableType::Text)
        .map(|v| v.clone())
        .collect();

    let mut words: HashMap<String, Locator> = HashMap::new();
    for var in variables {
        if let Some(occurrences) = context.frequency_table.get(&var) {
            for (val, _occ) in occurrences.iter() {
                let mut locator = Locator::from(&var);
                locator.value_index = val.row;

                words.insert(val.value.to_string(), locator.clone());
            }
        }
    }

    dictionary(context, StringValueSpellcheck, &words, spellcheck_predicate);
}

/// Notify if a variable has duplicate values, and where they are
fn duplicate_values(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.data_integrity.duplicate_values {
        use check::CheckName::DuplicateValues;
        include_check!(
            report.summary,
            DuplicateValues,
            format!("{} (On variables {:?})", setting.desc, setting.setting).as_str(),
            Category::DataIntegrity
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

fn string_value_odd_characters(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.data_integrity.string_value_odd_characters {
        use check::CheckName::StringValueOddCharacters;
        include_check!(
            report.summary,
            StringValueOddCharacters,
            format!("{} {:?}", setting.desc, &setting.setting).as_str(),
            Category::DataIntegrity
        );

        if let Some(ref mut status) = report.summary.get_mut(&StringValueOddCharacters) {
            for variable in (*context).variables.iter() {
                if let Some(values) = (*context).frequency_table.get(&variable) {
                    for (value, _occ) in values.iter().filter(|(v, _)| match v.value {
                        AnyValue::Str(_) => true,
                        _ => false,
                    }) {
                        if contains(&format!("{}", &value.value), &setting.setting) {
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

/// Report variables with a number of system missing values over a
/// specified threhold.
fn system_missing_over_threshold(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.data_integrity.system_missing_value_threshold {
        use check::CheckName::SystemMissingOverThreshold;
        include_check!(
            report.summary,
            SystemMissingOverThreshold,
            format!("{} (Threshold: {}%)", setting.desc, setting.setting).as_str(),
            Category::DataIntegrity
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

/// Flags values that match a regex pattern
fn regex_patterns(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.disclosure_risk.regex_patterns {
        use check::CheckName::ValueRegexPatterns;
        include_check!(
            report.summary,
            ValueRegexPatterns,
            &setting.desc,
            Category::DisclosureRisk
        );

        if let Some(ref mut status) = report.summary.get_mut(&ValueRegexPatterns) {
            for variable in context.variables.iter() {
                for (value, _occ) in context.frequency_table.get(&variable).unwrap() {
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
                            break;
                        }
                    }
                }
            }

            status.pass = report.metadata.variable_count - status.fail;
        }
    }
}

/// Count the number of variables with one or more unique values
fn unique_values(context: &mut Context) {
    let (config, report) = (&context.config, &mut context.report);

    if let Some(ref setting) = config.disclosure_risk.unique_values {
        use check::CheckName::VariablesWithUniqueValues;
        include_check!(
            report.summary,
            VariablesWithUniqueValues,
            &setting.desc,
            Category::DisclosureRisk
        );

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

fn string_value_stopword(context: &mut Context) {
    use check::CheckName::StringValueStopword;

    let variables: Vec<Variable> = context.variables.iter()
        .filter(|v| v.type_ == VariableType::Text)
        .map(|v| v.clone())
        .collect();

    let mut words: HashMap<String, Locator> = HashMap::new();
    for var in variables {
        if let Some(occurrences) = context.frequency_table.get(&var) {
            for (val, _occ) in occurrences.iter() {
                let mut locator = Locator::from(&var);
                locator.value_index = val.row;

                words.insert(val.value.to_string(), locator.clone());
            }
        }
    }

    //println!("words: {:#?}", &words);

    dictionary(context, StringValueStopword, &words, stopword_predicate);
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
            bar.label = String::from("this@ is far too long to pss the test");

            temp.insert(bar, 3);
            temp.insert(Value::from("!baz"), 3);
            temp.insert(qux, 4);

            let variable = Variable {
                index: 0,
                name: "first".to_string(),
                label: "first fine label".to_string(),
                type_: VariableType::Text,
                value_format: String::new(),
                value_labels: "labels1".to_string(),
            };

            freq_table.insert(variable, temp.clone());
        }

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            let mut missing_value: Value = Value::from("");
            missing_value.missing = Missing::SYSTEM_MISSING;

            temp.insert(Value::from("g@regs"), 2);
            temp.insert(missing_value, 8);

            let variable = Variable {
                index: 1,
                name: "second".to_string(),
                label: "second fine label".to_string(),
                type_: VariableType::Text,
                value_format: String::new(),
                value_labels: String::new(),
            };

            freq_table.insert(variable, temp);
        }

        {
            let mut temp: HashMap<Value, i32> = HashMap::new();
            let variable = Variable {
                index: 2,
                name: "badid".to_string(),
                label: "this is nt ok".to_string(),
                type_: VariableType::Numeric,
                value_format: String::new(),
                value_labels: String::new(),
            };

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
            let variable = Variable {
                index: 3,
                name: "id".to_string(),
                label: "this is nt ok either".to_string(),
                type_: VariableType::Numeric,
                value_format: String::new(),
                value_labels: String::new(),
            };

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

        let mut value_labels: HashMap<String, HashMap<String, String>> = HashMap::new();

        {
            let mut temp: HashMap<String, String> = HashMap::new();

            temp.insert("qux".to_string(),
                "this is fine".to_string());
            temp.insert("bar#".to_string(),
                "this@ is far too long to pss the test".to_string());

            value_labels.insert("labels1".to_string(), temp);
        }

        let mut report = Report::new();
        report.metadata.variable_count = 4;
        report.metadata.raw_case_count = 10;

        let variables = freq_table.keys().map(|v| v.clone()).collect();

        Context {
            config: Config::default(),
            report: report,
            checks: Check {
                variable: vec![],
                value: vec![],
                post: vec![],
            },
            pb: None,
            variables: variables,
            value_labels: value_labels,
            frequency_table: freq_table,
        }
    }

    #[test]
    fn test_bad_filename() {
        let mut context = setup();

        use check::CheckName::BadFileName;

        assert!(context.report.summary.get(&BadFileName).is_none());

        context.config.basic_file_checks.bad_filename = Some(Setting {
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

    #[test]
    fn test_primary_variable() {
        let mut context = setup();
        assert!(context.report.metadata.case_count.is_none());

        context.config.metadata.primary_variable = Some(Setting {
            setting: String::from("first"),
            desc: String::from("primary variable"),
        });

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

        context.config.data_integrity.duplicate_values = Some(Setting {
            setting: vec!["id", "badid"].iter().map(|s| s.to_string()).collect(),
            desc: "description from config".to_string(),
        });

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

        context.config.data_integrity.system_missing_value_threshold = Some(Setting {
            setting: 25,
            desc: String::from("sysmiss values over a threshold"),
        });

        system_missing_over_threshold(&mut context);
        assert_setting!(
            context.report.summary.get(&SystemMissingOverThreshold),
            3,
            1
        );
    }

    #[test]
    fn test_unique_values() {
        let mut context = setup();

        use check::CheckName::VariablesWithUniqueValues;

        assert!(context
            .report
            .summary
            .get(&VariablesWithUniqueValues)
            .is_none());

        context.config.disclosure_risk.unique_values = Some(Setting {
            setting: 2,
            desc: String::from("outliers as defined by the threshold"),
        });

        unique_values(&mut context);
        assert_setting!(context.report.summary.get(&VariablesWithUniqueValues), 1, 3);
    }

    #[test]
    fn test_value_label_max_length() {
        let mut context = setup();

        use check::CheckName::ValueLabelMaxLength;

        assert!(context.report.summary.get(&ValueLabelMaxLength).is_none());

        context.config.metadata.value_label_max_length = Some(Setting {
            setting: 20,
            desc: String::from("value labels cannot be too long"),
        });

        value_label_max_length(&mut context);
        assert_setting!(context.report.summary.get(&ValueLabelMaxLength), 3, 1);
    }

    #[test]
    fn test_value_label_odd_characters() {
        let mut context = setup();

        use check::CheckName::ValueLabelOddCharacters;

        assert!(context
            .report
            .summary
            .get(&ValueLabelOddCharacters)
            .is_none());

        context.config.metadata.value_label_odd_characters = Some(Setting {
            setting: vec!["#", "@", "!"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            desc: String::from("value labels shouldn't contain some characters"),
        });

        value_label_odd_characters(&mut context);
        assert_setting!(context.report.summary.get(&ValueLabelOddCharacters), 1, 1);
    }

    #[test]
    fn test_string_value_odd_characters() {
        let mut context = setup();

        use check::CheckName::StringValueOddCharacters;

        assert!(context
            .report
            .summary
            .get(&StringValueOddCharacters)
            .is_none());

        context.config.data_integrity.string_value_odd_characters = Some(Setting {
            setting: vec!["#", "@", "!"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            desc: String::from("description from config"),
        })
    }

    #[test]
    fn test_regex_patterns() {
        let mut context = setup();

        use check::CheckName::ValueRegexPatterns;

        assert!(context.report.summary.get(&ValueRegexPatterns).is_none());

        context.config.disclosure_risk.regex_patterns = Some(Setting {
            setting: vec![r"^qux".to_string()],
            desc: "description from config".to_string(),
        });

        regex_patterns(&mut context);
        assert_setting!(context.report.summary.get(&ValueRegexPatterns), 3, 1);
    }

    #[test]
    fn test_value_label_spellcheck() {
        let mut context = setup();

        use check::CheckName::ValueLabelSpellcheck;

        assert!(context.report.summary.get(&ValueLabelSpellcheck).is_none(),
            "ValueLabelSpellcheck was set in the summary report");

        context.config.metadata.value_label_spellcheck = Some(Setting {
            setting: vec!["test/words.txt".to_string()],
            desc: "spellcheck: description from config".to_string(),
        });

        value_label_spellcheck(&mut context);
        assert_setting!(context.report.summary.get(&ValueLabelSpellcheck), 1, 1);
    }

    #[test]
    fn test_variable_label_spellcheck() {
        let mut context = setup();
        use check::CheckName::VariableLabelSpellcheck;

        assert!(context.report.summary.get(&VariableLabelSpellcheck).is_none(),
            "VariableLabelSpellcheck was set in the summary report");

        context.config.metadata.variable_label_spellcheck = Some(Setting {
            setting: vec!["test/words.txt".to_string()],
            desc: "variable label spellcheck: description from config".to_string(),
        });

        variable_label_spellcheck(&mut context);
        assert_setting!(context.report.summary.get(&VariableLabelSpellcheck), 2, 2);
    }

    #[test]
    fn test_string_value_spellcheck() {
        let mut context = setup();

        use check::CheckName::StringValueSpellcheck;

        assert!(context.report.summary.get(&StringValueSpellcheck).is_none(),
            "StringValueSpellcheck was set in the summary report.");

        context.config.data_integrity.string_value_spellcheck = Some(Setting {
            setting: vec!["test/words.txt".to_string()],
            desc: "string value spellcheck: description from config".to_string(),
        });

        string_value_spellcheck(&mut context);
        assert_setting!(context.report.summary.get(&StringValueSpellcheck), 3, 2);
    }

    #[test]
    fn test_string_value_stopword() {
        let mut context = setup();

        use check::CheckName::StringValueStopword;

        assert!(context.report.summary.get(&StringValueStopword).is_none(),
            "StringValueStopword was set in the summary report.");

        context.config.disclosure_risk.string_value_stopword = Some(Setting {
            setting: vec!["test/stopwords.txt".to_string()],
            desc: "string value stopword: desc from config".to_string(),
        });

        string_value_stopword(&mut context);
        assert_setting!(context.report.summary.get(&StringValueStopword), 4, 1);
    }
}
