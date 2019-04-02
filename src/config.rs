use std::fs::File;
use std::io;
use std::io::prelude::*;

pub trait Valid {
    fn validate(&self) -> Result<(), &'static str>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FileType {
    SAV,
    DTA,
    SAS7BDAT,
    CSV,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Setting<T> {
    pub setting: T,
    pub desc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub include_locators: Option<bool>,
    pub progress: Option<bool>,

    pub spellcheck: Option<Setting<Vec<String>>>,

    pub variable_config: VariableConfig,
    pub value_config: ValueConfig,
}

impl Config {
    pub fn new() -> Config {
        Config {
            include_locators: None,
            progress: None,

            spellcheck: None,

            variable_config: VariableConfig::new(),
            value_config: ValueConfig::new(),
        }
    }

    pub fn get_dictonary(&self) -> Result<Vec<String>, String> {
        if let Some(ref paths) = self.spellcheck {
            let mut result: Vec<String> = vec![];

            for path in paths.setting.iter() {
                match Config::read_file(&path) {
                    Ok(contents) => result.extend(
                        contents
                            .split("\n")
                            .map(|s| s.trim().to_string())
                            .collect::<Vec<String>>(),
                    ),
                    Err(e) => return Err(e.to_string()),
                }
            }

            return Ok(result);
        }

        Err("No file specified.".to_string())
    }

    fn read_file(path: &str) -> io::Result<String> {
        let mut f = File::open(path)?;

        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;

        Ok(buffer)
    }
}

impl Valid for Config {
    fn validate(&self) -> Result<(), &'static str> {
        self.variable_config.validate()?;
        self.value_config.validate()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariableConfig {
    pub odd_characters: Option<Setting<Vec<String>>>,
    pub missing_variable_labels: Option<Setting<bool>>,
    pub label_max_length: Option<Setting<i32>>,

    pub date_format: Option<Setting<Vec<String>>>,

    pub primary_variable: Option<Setting<String>>,
    pub variables_with_unique_values: Option<Setting<i32>>,
}

impl VariableConfig {
    pub fn new() -> VariableConfig {
        VariableConfig {
            primary_variable: None,
            variables_with_unique_values: None,

            odd_characters: None,
            missing_variable_labels: None,
            label_max_length: None,

            date_format: None,
        }
    }
}

impl Valid for VariableConfig {
    fn validate(&self) -> Result<(), &'static str> {
        match self.primary_variable {
            None => (),
            Some(ref primary_variable) => {
                if primary_variable.setting.len() < 1 {
                    return Err("primary_variable cannot be an empty string");
                }
            }
        }

        match self.variables_with_unique_values {
            None => (),
            Some(ref threshold) => {
                if !(threshold.setting > 0 && threshold.setting <= 100) {
                    return Err("threshold out of bounds");
                }
            }
        }

        match self.odd_characters {
            None => (),
            Some(ref odd_characters) => {
                if odd_characters.setting.len() < 1 {
                    return Err("variable_config.odd_characters cannot be empty");
                }
            }
        }

        match self.label_max_length {
            None => (),
            Some(ref label_max_length) => {
                if label_max_length.setting < 0 {
                    return Err("variable_config.label_max_length cannot be negative");
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValueConfig {
    pub odd_characters: Option<Setting<Vec<String>>>,
    pub system_missing_value_threshold: Option<Setting<i32>>,
    pub label_max_length: Option<Setting<i32>>,
    pub defined_missing_no_label: Option<Setting<bool>>,
    pub regex_patterns: Option<Setting<Vec<String>>>,
}

impl ValueConfig {
    pub fn new() -> ValueConfig {
        ValueConfig {
            odd_characters: None,
            system_missing_value_threshold: None,
            defined_missing_no_label: None,
            label_max_length: None,
            regex_patterns: None,
        }
    }
}

impl Valid for ValueConfig {
    fn validate(&self) -> Result<(), &'static str> {
        match self.odd_characters {
            None => (),
            Some(ref odd_characters) => {
                if odd_characters.setting.len() < 1 {
                    return Err("value_config.odd_characters cannot be empty");
                }
            }
        }

        match self.label_max_length {
            None => (),
            Some(ref label_max_length) => {
                if label_max_length.setting < 0 {
                    return Err("value_config.label_max_length cannot be negative");
                }
            }
        }

        match self.system_missing_value_threshold {
            None => (),
            Some(ref threshold) => {
                if !(threshold.setting > 0 && threshold.setting <= 100) {
                    return Err("threshold out of bounds");
                }
            }
        }

        Ok(())
    }
}
