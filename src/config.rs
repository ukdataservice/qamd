use std::path::Path;

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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    pub metadata_only: Option<bool>,
    pub progress: Option<bool>,

    pub basic_file_checks: BasicFileChecks,
    pub metadata: Metadata,
    pub data_integrity: DataIntegrity,
    pub disclosure_risk: DisclosureRisk,
}

impl Config {
    pub fn get_dictionaries(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];

        if let Some(ref paths) = self.metadata.spellcheck {
            for spath in paths.setting.iter() {
                let path = Path::new(spath);

                if path.is_file() {
                    result.push(
                        path.to_str()
                            .expect("Failed to convert path to str.")
                            .to_string(),
                    );
                }
            }
        }

        return result;
    }
}

impl Valid for Config {
    fn validate(&self) -> Result<(), &'static str> {
        self.basic_file_checks.validate()?;
        self.metadata.validate()?;
        self.data_integrity.validate()?;
        self.disclosure_risk.validate()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BasicFileChecks {
    pub bad_filename: Option<Setting<String>>,
}

impl Valid for BasicFileChecks {
    fn validate(&self) -> Result<(), &'static str> {
        match self.bad_filename {
            None => (),
            Some(ref pattern) => {
                if pattern.setting.len() < 1 {
                    return Err("bad_filename cannot be an empty string");
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Metadata {
    pub primary_variable: Option<Setting<String>>,

    pub missing_variable_labels: Option<Setting<bool>>,
    pub variable_odd_characters: Option<Setting<Vec<String>>>,
    pub variable_label_max_length: Option<Setting<i32>>,

    pub value_label_odd_characters: Option<Setting<Vec<String>>>,
    pub value_label_max_length: Option<Setting<i32>>,

    pub spellcheck: Option<Setting<Vec<String>>>,
    pub value_defined_missing_no_label: Option<Setting<bool>>, // SPSS only. E.g. -9 is Defined missing but has no label
}

impl Valid for Metadata {
    fn validate(&self) -> Result<(), &'static str> {
        match self.primary_variable {
            None => (),
            Some(ref primary_variable) => {
                if primary_variable.setting.len() < 1 {
                    return Err("metadata.primary_variable cannot be an empty string");
                }
            }
        }

        match self.variable_odd_characters {
            None => (),
            Some(ref odd_characters) => {
                if odd_characters.setting.len() < 1 {
                    return Err("metadata.variable_odd_characters cannot be empty");
                }
            }
        }

        match self.variable_label_max_length {
            None => (),
            Some(ref label_max_length) => {
                if label_max_length.setting < 0 {
                    return Err("metadata.variable_label_max_length cannot be negative");
                }
            }
        }

        match self.value_label_odd_characters {
            None => (),
            Some(ref odd_characters) => {
                if odd_characters.setting.len() < 1 {
                    return Err("metadata.value_label_odd_characters cannot be empty");
                }
            }
        }

        match self.value_label_max_length {
            None => (),
            Some(ref label_max_length) => {
                if label_max_length.setting < 0 {
                    return Err("metadata.value_label_max_length cannot be negative");
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DataIntegrity {
    pub duplicate_values: Option<Setting<Vec<String>>>,

    pub string_value_odd_characters: Option<Setting<Vec<String>>>,
    pub system_missing_value_threshold: Option<Setting<i32>>,
}

impl Valid for DataIntegrity {
    fn validate(&self) -> Result<(), &'static str> {
        match self.duplicate_values {
            None => (),
            Some(ref variables) => {
                if variables.setting.len() < 1 {
                    return Err("data_integrity.duplicate_values cannot be empty");
                }
            }
        }

        match self.string_value_odd_characters {
            None => (),
            Some(ref odd_characters) => {
                if odd_characters.setting.len() < 1 {
                    return Err("data_integrity.string_value_odd_characters cannot be empty");
                }
            }
        }

        match self.system_missing_value_threshold {
            None => (),
            Some(ref threshold) => {
                if !(threshold.setting > 0 && threshold.setting <= 100) {
                    return Err("data_integrity.system_missing_value_threshold out of bounds, must be between 1 and 100 inclusive");
                }
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DisclosureRisk {
    pub date_format: Option<Setting<Vec<String>>>,

    pub regex_patterns: Option<Setting<Vec<String>>>,
    pub unique_values: Option<Setting<i32>>,
}

impl Valid for DisclosureRisk {
    fn validate(&self) -> Result<(), &'static str> {
        match self.regex_patterns {
            None => (),
            Some(ref patterns) => {
                if patterns.setting.len() < 1 {
                    return Err("data_integrity.regex_patterns cannot be empty");
                }
            }
        }

        match self.unique_values {
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
