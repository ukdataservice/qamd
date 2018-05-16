
trait Valid {
    fn validate(&self) -> Result<(), &'static str>;
}

#[derive(Deserialize, Clone, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
pub struct VariableConfig {
    pub odd_characters: Option<Vec<String>>,
    pub missing_variable_labels: bool,
}

impl Valid for VariableConfig {
    fn validate(&self) -> Result<(), &'static str> {
        match self.odd_characters {
            None => (),
            Some(ref odd_characters) => if odd_characters.len() < 1 {
                return Err("variable_config.odd_characters cannot be empty");
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ValueConfig {
    pub odd_characters: Option<Vec<String>>,
    pub system_missing_value_threshold: Option<i32>,
}

impl Valid for ValueConfig {
    fn validate(&self) -> Result<(), &'static str>{
        match &self.odd_characters {
            &None => (),
            &Some(ref odd_characters) => if odd_characters.len() < 1 {
                return Err("value_config.odd_characters cannot be empty");
            }
        }

        match self.system_missing_value_threshold {
            None => (),
            Some(threshold) => if !(threshold > 0 && threshold <= 100) {
                return Err("threshold out of bounds");
            }
        }

        Ok(())
    }
}

