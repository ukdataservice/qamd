
#[derive(Deserialize, Debug)]
pub struct Config {
    pub file_encoding: bool,
    pub odd_characters: Vec<String>,
    pub missing_variable_labels: bool,
    pub system_missing_value_threshold: Option<i32>,
}

impl Config {
    pub fn validate(&self) -> Result<(), &'static str>{
        match self.system_missing_value_threshold {
            None => (),
            Some(threshold) => if !(threshold > 0 && threshold <= 100) {
                return Err("threshold out of bounds");
            }
        }

        Ok(())
    }
}

