use check::{CheckName, read_file, only_contains};
use readstat::context::Context;
use report::{Category, Status, Locator};

use std::collections::{HashMap, HashSet};

pub fn dictionary(context: &mut Context,
                  check_name: CheckName,
                  words: &HashMap<String, Locator>) {
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

    //let normalized_words: HashMap<String, Locator> = words.iter()

    if let Some(ref mut status) = report.summary.get_mut(&check_name) {
        for (word, locator) in words.iter()
            .map(|(k, v)| { (normalize_word(k), v) }) {
            if word.is_empty() || only_contains(&word, &dictonary) {
                status.pass += 1;
            } else {
                if let Some(metadata_only) = config.metadata_only {
                    if !metadata_only {
                        if let Some(ref mut locators) = status.locators {
                            locators.insert(locator.clone());
                        } else {
                            let mut set = HashSet::new();
                            set.insert(locator.clone());
                            status.locators = Some(set);
                        }
                    }
                }

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

