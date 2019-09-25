extern crate qamd;
extern crate serde_yaml;

use qamd::config::*;

fn main() -> Result<(), serde_yaml::Error> {
    let odd_chars = vec_of_strings(vec!["!", "#", "  ", "@", "ë", "ç", "ô", "ü"]);
    let dicts = vec_of_strings(vec![
        "/usr/share/dict/words",
        "C:\\path\\to\\dictonary\\file.txt",
    ]);
    let duplicate_values = vec_of_strings(vec!["Caseno"]);
    let regexps = vec_of_strings(vec![
        "^([\\w\\.\\-]+)@([\\w\\-]+)((\\.(\\w){2,4})+)$",
        "([Gg][Ii][Rr] 0[Aa]{2})|((([A-Za-z][0-9]{1,2})|(([A-Za-z][A-Ha-hJ-Yj-y][0-9]{1,2})|(([A-Za-z][0-9][A-Za-z])|([A-Za-z][A-Ha-hJ-Yj-y][0-9]?[A-Za-z]))))\\s?[0-9][A-Za-z]{2})",
        "((([a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(\\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*)|(\"(([\\x01-\\x08\\x0B\\x0C\\x0E-\\x1F\\x7F]|[\\x21\\x23-\\x5B\\x5D-\\x7E])|(\\[\\x01-\\x09\\x0B\\x0C\\x0E-\\x7F]))*\"))@(([a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(\\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*)|(\\[(([\\x01-\\x08\\x0B\\x0C\\x0E-\\x1F\\x7F]|[\\x21-\\x5A\\x5E-\\x7E])|(\\[\\x01-\\x09\\x0B\\x0C\\x0E-\\x7F]))*\\])))",
    ]);

    let config = Config {
            metadata_only: None,
            progress: None,


            basic_file_checks: BasicFileChecks {
                bad_filename: Some(setting(
                                      r#"^([a-zA-Z0-9]+)\.([a-zA-Z0-9]+)$"#.to_string(),
                                      "Filenames must match a given regular expression to be considered valid."
                                   )),
            },
            metadata: Metadata {
                primary_variable: Some(setting(
                                          "Caseno".to_string(),
                                          "Counts the unqiue occurences for the variable, useful if your dataset groups by household."
                                       )),

                missing_variable_labels: Some(setting(
                                                true,
                                                "Variables should have a label."
                                            )),
                variable_odd_characters: Some(setting(odd_chars.clone(), "Variable names and lables cannot contain certain 'odd' characters.")),
                variable_label_max_length: Some(setting(79, "Variable labels cannot exceed a maximum length.")),
                variable_label_spellcheck: Some(setting(dicts.clone(), "Word file(s) used for spellchecking variable labels.")),

                value_label_odd_characters: Some(setting(odd_chars.clone(), "Value labels cannot contain certain 'odd' characters")),
                value_label_max_length: Some(setting(39, "Value labels cannot exceet a maximum length")),
                value_label_spellcheck: Some(setting(dicts.clone(), "Word file(s) used for spellchecking value labels.")),

                value_defined_missing_no_label: Some(setting(true, "Values defined as missing must have a label (only applicable to SPSS data files)")),
            },
            data_integrity: DataIntegrity {
                duplicate_values: Some(setting(duplicate_values, "For each variable specified will check for duplicate values. Useful for checking all ID's are unique.")),

                string_value_odd_characters: Some(setting(odd_chars, "String values cannot contain certain 'odd' characters.")),
                string_value_spellcheck: Some(setting(dicts, "Word file(s) used for spellchecking string values.")),
                system_missing_value_threshold: Some(setting(25, "Percentage of missing variables that becomes unacceptable.")),
            },
            disclosure_risk: DisclosureRisk {
                date_format: None,

                regex_patterns: Some(setting(regexps, "Values matching a regex pattern fail. Can be used to find post codes and telephone numbers.")),
                unique_values: Some(setting(1, "Detects outliers (if a variable contains unique values)")),
            }
    };

    if config.validate().is_ok() {
        let desrialized = serde_yaml::to_string(&config)?;
        println!("{}", desrialized);
    }

    Ok(())
}

fn vec_of_strings<'a>(v: Vec<&'a str>) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect::<Vec<String>>()
}

fn setting<'a, T>(t: T, s: &'a str) -> Setting<T> {
    Setting {
        setting: t,
        desc: s.to_string(),
    }
}
