
pub mod anyvalue;
pub mod missing;

use self::anyvalue::AnyValue;
use self::missing::Missing;

// use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub variable_checks: VariableChecks,
    pub value_checks: ValueChecks,
}

impl Report {
    pub fn new() -> Report {
        Report {
            metadata: Metadata::new(),
            variable_checks: VariableChecks::new(),
            value_checks: ValueChecks::new(),
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

#[derive(Serialize, Debug, Clone)]
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

