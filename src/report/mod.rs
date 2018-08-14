
pub mod anyvalue;
pub mod missing;

use self::anyvalue::AnyValue;
use self::missing::Missing;

use readstat::bindings::*;

use std::hash::{ Hash, Hasher };
use std::iter::Iterator;

use std::os::raw::c_char;
use std::ffi::{ /*CString,*/ CStr };

use std::ptr;

// use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub summary: Summary,
}

impl Report {
    pub fn new() -> Report {
        Report {
            metadata: Metadata::new(),
            summary: Summary::new(),
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
pub struct Summary {
    // counting variables that failed
    pub missing_variable_labels: Option<Status>,
    pub variable_label_max_length: Option<Status>,
    pub variable_odd_characters: Option<Status>,

    // counting values that failed
    pub value_label_max_length: Option<Status>,
    pub value_odd_characters: Option<Status>,
    pub value_defined_missing_no_label: Option<Status>,

    // post checks
    pub system_missing_over_threshold: Option<Status>, // number of variables
    pub variables_with_unique_values: Option<Status>, // number of variables
    pub date_format: Option<Status>, // number of variables
}

pub struct SummaryIntoIterator {
    summary: Summary,
    index: usize,
}

impl Summary {
    pub fn new() -> Summary {
        Summary {
            missing_variable_labels: None,
            variable_label_max_length: None,
            variable_odd_characters: None,

            value_label_max_length: None,
            value_odd_characters: None,
            value_defined_missing_no_label: None,

            system_missing_over_threshold: None,
            variables_with_unique_values: None,
            date_format: None,
        }
    }
}

impl IntoIterator for Summary {
    type Item = (String, Option<Status>);
    type IntoIter = SummaryIntoIterator;

    fn into_iter(self) -> SummaryIntoIterator {
        SummaryIntoIterator {
            summary: self,
            index: 0,
        }
    }
}

impl Iterator for SummaryIntoIterator {
    type Item = (String, Option<Status>);

    // TODO: better system for iterating structs
    fn next(&mut self) -> Option<(String, Option<Status>)> {
        let result = match self.index {
            0 => ("missing variable labels".into(), self.summary.missing_variable_labels.clone()),
            1 => ("variable label max length".into(), self.summary.variable_label_max_length.clone()),
            2 => ("variable odd characters".into(), self.summary.variable_odd_characters.clone()),
            3 => ("date format".into(), self.summary.date_format.clone()),

            4 => ("value label max length".into(), self.summary.value_label_max_length.clone()),
            5 => ("value odd characters".into(), self.summary.value_odd_characters.clone()),
            6 => ("value defined missing no label".into(), self.summary.value_defined_missing_no_label.clone()),

            7 => ("system missing over threshold".into(), self.summary.system_missing_over_threshold.clone()),
            8 => ("variables with unique values".into(), self.summary.variables_with_unique_values.clone()),
            _ => return None,
        };

        self.index += 1;
        Some(result)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Status {
    pub pass: i32,
    pub fail: i32,
    pub desc: String,
    pub locator: Option<Vec<Locator>>,
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

#[derive(Serialize, Debug, Clone)]
pub struct Locator {
    pub variable_name: String,
    pub variable_index: i32,
    pub value_index: i32,
}

impl Locator {
    pub fn new(variable_name: String,
               variable_index: i32,
               value_index: i32) -> Locator {
        Locator {
            variable_name: variable_name,
            variable_index: variable_index,
            value_index: value_index,
        }
    }
}

#[derive(Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Variable {
    pub index: i32,
    pub name: String,
    pub label: String,
    pub value_format: String,
    pub value_labels: String,
}

impl Variable {
    pub fn from_raw_parts(variable: *mut readstat_variable_s,
                          val_labels: *const c_char) -> Self {
        unsafe {
            let index = readstat_variable_get_index(variable);

            let variable_name = ptr_to_str!(readstat_variable_get_name(variable));

            let label = if readstat_variable_get_label(variable) != ptr::null() {
                ptr_to_str!(readstat_variable_get_label(variable))
            } else {
                String::new()
            };

            let value_format = if readstat_variable_get_format(variable) != ptr::null() {
                ptr_to_str!(readstat_variable_get_format(variable))
            } else {
                String::new()
            };

            let value_labels = if val_labels != ptr::null() {
                ptr_to_str!(val_labels)
            } else {
                "".into()
            };

            Variable {
                index: index as i32,
                name: variable_name,
                label: label,
                value_format: value_format,
                value_labels: value_labels,
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Value {
    pub variable: Variable,
    pub row: i32,
    pub value: AnyValue,
    pub label: String,
    pub missing: Missing,
}

/// Hash implemtation distiguishes values based on `value` field ONLY
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

