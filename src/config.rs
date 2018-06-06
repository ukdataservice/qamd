
trait Valid {
    fn validate(&self) -> Result<(), &'static str>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Level {
    Pass,
    Warn,
    Fail,
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
    pub level: Level,
    pub file_types: Vec<FileType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub variable_config: VariableConfig,
    pub value_config: ValueConfig,
}

impl Valid for Config {
    fn validate(&self) -> Result<(), &'static str> {

        self.variable_config.validate()?;
        self.variable_config.validate()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VariableConfig {
    pub odd_characters: Setting<Option<Vec<String>>>,
    pub missing_variable_labels: Setting<bool>,
}

impl Valid for VariableConfig {
    fn validate(&self) -> Result<(), &'static str> {
        match self.odd_characters.setting {
            None => (),
            Some(ref odd_characters) => if odd_characters.len() < 1 {
                return Err("variable_config.odd_characters cannot be empty");
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValueConfig {
    pub odd_characters: Setting<Option<Vec<String>>>,
    pub system_missing_value_threshold: Setting<Option<i32>>,
}

impl Valid for ValueConfig {
    fn validate(&self) -> Result<(), &'static str>{
        match &self.odd_characters.setting {
            &None => (),
            &Some(ref odd_characters) => if odd_characters.len() < 1 {
                return Err("value_config.odd_characters cannot be empty");
            }
        }

        match self.system_missing_value_threshold.setting {
            None => (),
            Some(threshold) => if !(threshold > 0 && threshold <= 100) {
                return Err("threshold out of bounds");
            }
        }

        Ok(())
    }
}

