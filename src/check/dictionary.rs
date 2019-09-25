use check::{CheckName, read_file, only_contains};
use readstat::context::Context;
use report::{Category, Status};

pub fn dictionary(context: &mut Context,
                  check_name: CheckName,
                  words: &Vec<String>) {
    let (config, report) = (&context.config, &mut context.report);

    // validate that CheckName is a spellcheck
    match check_name {
        CheckName::VariableLabelSpellcheck => {
            if config.metadata.variable_label_spellcheck.is_none() {
                return;
            }
        },
        CheckName::ValueLabelSpellcheck => {
            if config.metadata.value_label_spellcheck.is_none() {
                return;
            }
        },
        CheckName::StringValueSpellcheck => {
            if config.data_integrity.string_value_spellcheck.is_none() {
                return;
            }
        },
        _ => return,
    }

    // get the description
    let setting_desc = config.get_spellcheck_desc(&check_name);

    // get the dictonaries paths
    let dictonaries_paths = config.get_dictionaries(&check_name);

    // build the dictionary by stitching together the files into a vec
    let dictonary: Vec<String> = dictonaries_paths
        .iter()
        .map(|path| read_file(path))
        .filter_map(|result| result.ok())
        .map(|s| {
            s.split("\n")
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect();

    let category = if check_name == CheckName::StringValueSpellcheck {
        Category::DataIntegrity
    } else {
        Category::Metadata
    };

    include_check!(
        report.summary,
        check_name.clone(),
        &setting_desc,
        category
    );

    let normalized_words: Vec<String> = words.iter()
        .map(normalize_word)
        .collect();

    if let Some(ref mut status) = report.summary.get_mut(&check_name) {
        for word in normalized_words.iter() {
            if word.is_empty() || only_contains(&word, &dictonary) {
                status.pass += 1;
            } else {
                //include_locators!(
                //    config,
                //    status,
                //    value.variable.name,
                //    value.variable.index,
                //    value.row
                //);

                status.fail += 1;
            }
        }

        {
            let total_values = words.len();
            let total_counted = status.pass + status.fail;
            assert!(
                total_counted == total_values as i32,
                "Total counted: {} is not equal to total values: {}",
                total_counted,
                total_values
            );
        }
    }
}

fn normalize_word(word: &String) -> String {
    word.chars()
        .filter(|c| !char::is_ascii_punctuation(c))
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    use check::Check;
    use config::{Config, Setting};
    use model::anyvalue::AnyValue;
    use model::value::Value;
    use model::variable::Variable;
    use model::missing::Missing;
    use report::Report;

    use std::collections::HashMap;

    fn setup() -> (Context, Vec<String>) {

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

            let mut first = Variable::from("first");

            first.value_labels = "labels1".to_string();

            freq_table.insert(first, temp.clone());
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

        (Context {
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
        }, vec!["pass".to_string(), "fial".to_string()])
    }

    #[test]
    fn test_dictonary_with_value_label_spellcheck() {
        let (mut context, words)= setup();
        use check::CheckName::ValueLabelSpellcheck;

        assert!(context.report.summary.get(&ValueLabelSpellcheck).is_none(),
            "ValueLabelSpellcheck was set in the summary report");

        context.config.metadata.value_label_spellcheck = Some(Setting {
            setting: vec!["test/words.txt".to_string()],
            desc: "spellcheck: description from config".to_string(),
        });

        //dictionary(&mut context, ValueLabelSpellcheck, &vec![]);
        //assert_setting!(context.report.summary.get(&ValueLabelSpellcheck), 0, 0);

        dictionary(&mut context, ValueLabelSpellcheck, &words);
        assert_setting!(context.report.summary.get(&ValueLabelSpellcheck), 1, 1);
    }
}

