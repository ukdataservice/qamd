use check::CheckName;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub summary: HashMap<CheckName, Status>,
}

impl Report {
    pub fn new() -> Report {
        Report {
            metadata: Metadata::new(),
            summary: HashMap::new(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    pub file_name: String,

    pub raw_case_count: i32,
    pub case_count: Option<i32>,
    pub variable_count: i32,

    pub creation_time: i64,
    pub modified_time: i64,

    pub file_label: String,
    pub file_format_version: i64,
    pub file_encoding: Option<String>,

    pub compression: String,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            file_name: "".into(),
            raw_case_count: 0,
            case_count: None,
            variable_count: 0,
            creation_time: 0,
            modified_time: 0,
            file_label: "".into(),
            file_format_version: 0,
            file_encoding: None,
            compression: "".into(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Status {
    pub pass: i32,
    pub fail: i32,
    pub desc: String,
    pub locator: Option<HashSet<Locator>>,
}

impl Status {
    pub fn new(desc: &str) -> Status {
        Status {
            pass: 0,
            fail: 0,
            desc: desc.to_string(),
            locator: None,
        }
    }
}

#[derive(Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Locator {
    pub variable_name: String,
    pub variable_index: i32,
    pub value_index: i32,
    pub reason: Option<String>,
}

impl Locator {
    pub fn new(variable_name: String,
               variable_index: i32,
               value_index: i32,
               reason: Option<String>) -> Locator {
        Locator {
            variable_name: variable_name,
            variable_index: variable_index,
            value_index: value_index,
            reason: reason,
        }
    }
}
