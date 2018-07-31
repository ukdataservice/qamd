
use config::Config;

use report::Report;
use report::{ Variable, Value };

use check::Check;

use std::collections::HashMap;
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

