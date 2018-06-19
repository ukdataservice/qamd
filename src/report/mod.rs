
pub mod anyvalue;
pub mod missing;

use std::hash::{ Hash, Hasher };

use self::anyvalue::AnyValue;
use self::missing::Missing;

// use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub summary: Summary,
    // pub variable_checks: VariableChecks,
    // pub value_checks: ValueChecks,
}

impl Report {
    pub fn new() -> Report {
        Report {
            metadata: Metadata::new(),
            summary: Summary::new(),
            // variable_checks: VariableChecks::new(),
            // value_checks: ValueChecks::new(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    pub raw_case_count: i32,
    pub case_count: Option<i32>,
    pub variable_count: i32,

    pub creation_time: i64,
    pub modified_time: i64,

    pub file_label: String,
    pub file_format_version: i64,
    pub file_encoding: Option<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            raw_case_count: 0,
            case_count: None,
            variable_count: 0,
            creation_time: 0,
            modified_time: 0,
            file_label: "".into(),
            file_format_version: 0,
            file_encoding: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Summary {
    // counting variables that failed
    pub variable_label_missing: Option<Status>,
    pub variable_label_max_length: Option<Status>,
    pub variable_odd_characters: Option<Status>,

    // counting values that failed
    pub value_label_max_length: Option<Status>,
    pub value_odd_characters: Option<Status>,
    pub value_defined_missing_no_label: Option<Status>,

    // post checks
    pub system_missing_over_threshold: Option<Status>, // number of variables
    pub disclosive_outliers: Option<Status>, // number of variables
}

impl Summary {
    pub fn new() -> Summary {
        Summary {
            variable_label_missing: None,
            variable_label_max_length: None,
            variable_odd_characters: None,

            value_label_max_length: None,
            value_odd_characters: None,
            value_defined_missing_no_label: None,

            system_missing_over_threshold: None,
            disclosive_outliers: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Status {
    pub pass: i32,
    //pub warn: i32,
    pub fail: i32,
}

impl Status {
    pub fn new() -> Status {
        Status {
            pass: 0,
            //warn: 0,
            fail: 0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct VariableChecks {
    pub odd_characters: Option<Vec<Variable>>,
    pub missing_variable_labels: Option<Vec<Variable>>,
    pub label_max_length: Option<Vec<Variable>>,
}

impl VariableChecks {
    pub fn new() -> VariableChecks {
        VariableChecks {
            odd_characters: None,
            missing_variable_labels: None,
            label_max_length: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ValueChecks {
    pub odd_characters: Option<Vec<Value>>,
    pub defined_missing_no_label: Option<Vec<Value>>,
    pub label_max_length: Option<Vec<Value>>,
}

impl ValueChecks {
    pub fn new() -> ValueChecks {
        ValueChecks {
            odd_characters: None,
            defined_missing_no_label: None,
            label_max_length: None,
        }
    }
}

#[derive(Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Variable {
    pub index: i32,
    pub name: String,
    pub label: String,
    pub value_labels: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Value {
    pub var_index: i32,
    pub row: i32,
    pub value: AnyValue,
    pub label: String,
    pub missing: Missing,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.value.eq(&other.value)
    }
}

impl Eq for Value {}

