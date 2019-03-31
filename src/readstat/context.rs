use config::Config;
use model::value::Value;
use model::variable::Variable;
use report::Report;

use check::Check;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::io;

use pbr::ProgressBar;

pub struct Context {
    pub config: Config,
    pub report: Report,
    pub checks: Check,
    pub pb: Option<ProgressBar<io::Stdout>>,
    pub variables: Vec<Variable>, // used for post-processing and iter'ing unordered hashmap
    pub value_labels: HashMap<String, HashMap<String, String>>, // used for getting value labels
    pub frequency_table: HashMap<Variable, HashMap<Value, i32>>,
}

impl Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Context")
            .field("config", &self.config)
            .field("report", &self.report)
            .field("checks", &self.checks)
            .field("variables", &self.variables)
            .field("value_labels", &self.value_labels)
            .field("frequency_table", &self.frequency_table)
            .finish()
    }
}
