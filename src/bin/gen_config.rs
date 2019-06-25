
extern crate qamd;
extern crate serde_yaml;

use qamd::config::{Config, Setting, VariableConfig, ValueConfig, Valid};

fn main() -> Result<(), serde_yaml::Error>{
    let odd_chars = vec_of_strings(vec!["!", "#", "  ", "@", "ë", "ç", "ô", "ü"]);
    let dicts = vec_of_strings(vec!["/usr/share/dict/words", "C:\\path\\to\\dictonary\\file.txt"]);
    let duplicate_values = vec_of_strings(vec!["Caseno"]);
    let regexps = vec_of_strings(vec![
        "^([\\w\\.\\-]+)@([\\w\\-]+)((\\.(\\w){2,4})+)$",
        "([Gg][Ii][Rr] 0[Aa]{2})|((([A-Za-z][0-9]{1,2})|(([A-Za-z][A-Ha-hJ-Yj-y][0-9]{1,2})|(([A-Za-z][0-9][A-Za-z])|([A-Za-z][A-Ha-hJ-Yj-y][0-9]?[A-Za-z]))))\\s?[0-9][A-Za-z]{2})",
        "((([a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(\\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*)|(\"(([\\x01-\\x08\\x0B\\x0C\\x0E-\\x1F\\x7F]|[\\x21\\x23-\\x5B\\x5D-\\x7E])|(\\[\\x01-\\x09\\x0B\\x0C\\x0E-\\x7F]))*\"))@(([a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+(\\.[a-zA-Z0-9!#$%&'*+/=?^_`{|}~-]+)*)|(\\[(([\\x01-\\x08\\x0B\\x0C\\x0E-\\x1F\\x7F]|[\\x21-\\x5A\\x5E-\\x7E])|(\\[\\x01-\\x09\\x0B\\x0C\\x0E-\\x7F]))*\\])))",
    ]);

    let config = Config {
            metadata_only: None,
            progress: None,

            spellcheck: Some(setting(dicts, "")),
            bad_filename: Some(setting(r#"^([a-zA-Z0-9]+)\.([a-zA-Z0-9]+)$"#.to_string(), "")),

            variable_config: Some(VariableConfig {
                primary_variable: Some(setting("Caseno".to_string(), "")),
                variables_with_unique_values: Some(setting(1, "")),

                odd_characters: Some(setting(odd_chars.clone(), "")),
                missing_variable_labels: Some(setting(true, "")),
                label_max_length: Some(setting(79, "")),

                date_format: None,
            }),
            value_config: Some(ValueConfig {
                odd_characters: Some(setting(odd_chars, "")),
                system_missing_value_threshold: Some(setting(25, "")),

                defined_missing_no_label: Some(setting(true, "")),
                label_max_length: Some(setting(39, "")),

                regex_patterns: Some(setting(regexps, "")),
                duplicate_values: Some(setting(duplicate_values, "")),
            }),
    };

    if config.validate().is_ok() {
        let desrialized = serde_yaml::to_string(&config)?;
        println!("{}", desrialized);
    }

    Ok(())
}

fn vec_of_strings<'a>(v: Vec<&'a str>) -> Vec<String>{
    v
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}

fn setting<'a, T>(t: T, s: &'a str) -> Setting<T> {
    Setting {
        setting: t,
        desc: s.to_string(),
    }
}

