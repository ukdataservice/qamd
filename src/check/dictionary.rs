use check::{CheckName, read_file};
use readstat::context::Context;
use report::{Category, Status, Locator};

use std::collections::{HashMap, HashSet};

/// Predicate to pass to the dictionary function.
/// True value denotes the word passed the check
/// False value denotes the word failed to the check
type DictionaryPredicate = fn(word: &str, dictionary: &Vec<String>) -> bool;

/// Spellcheck predicate accepts a word and dictionary list.
/// Returns true if the word is empty or contains only whitespace seperated
/// words that exist in the supplied dictionary
pub fn spellcheck_predicate(word: &str, dictionary: &Vec<String>) -> bool {
    word.is_empty() || only_contains(&word, dictionary)
}

/// Spellcheck predicate accepts a word and dictionary list.
/// Returns true if the word is empty or contains only whitespace seperated
/// words that do not exist in the supplied dictionary
pub fn stopword_predicate(word: &str, dictionary: &Vec<String>) -> bool {
    word.is_empty() || !only_contains(word, dictionary)
}

/// Performs checks agains a list of words and a dictionary based on predicate
/// test. If the predicate returns true, the
pub fn dictionary(context: &mut Context,
                  check_name: CheckName,
                  words: &HashMap<String, Locator>,
                  predicate: DictionaryPredicate) {
    let (config, report) = (&context.config, &mut context.report);

    // validate that CheckName is a spellcheck
    match config.config_for_check(&check_name).is_none() {
        true => return,
        false => (),
    }

    // get the description
    let setting_desc = config.get_desc(&check_name);

    // get the dictonaries paths
    let dictonaries_paths = config.get_dictionaries(&check_name);

    // build the dictionary by stitching together the files into a vec
    let dictionary: Vec<String> = dictonaries_paths
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
    } else if check_name == CheckName::StringValueStopword {
        Category::DisclosureRisk
    } else {
        Category::Metadata
    };

    include_check!(
        report.summary,
        check_name.clone(),
        &setting_desc,
        category
    );

    if let Some(ref mut status) = report.summary.get_mut(&check_name) {
        for (word, locator) in words.iter()
            .map(|(k, v)| { (normalize_word(k), v) }) {

            if predicate(&word, &dictionary) {
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

pub fn only_contains(string: &str, patterns: &Vec<String>) -> bool {
    string
        .split(" ")
        .map(|w| patterns.contains(&w.to_string()))
        .fold(true, |a, b| a && b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_contains() {
        let patterns = vec!["foo", "baz", "qux"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(only_contains("foo baz qux", &patterns));
        assert_eq!(only_contains("foo bar baz", &patterns), false);
    }
}

