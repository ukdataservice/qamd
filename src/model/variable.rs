use readstat::bindings::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

#[derive(Serialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Variable {
    pub index: i32,
    pub name: String,
    pub label: String,
    pub type_: VariableType,
    pub value_format: String,
    pub value_labels: String,
}

impl Variable {
    pub fn from_raw_parts(variable: *mut readstat_variable_s, val_labels: *const c_char) -> Self {
        unsafe {
            let index = readstat_variable_get_index(variable);

            let variable_name = ptr_to_str!(readstat_variable_get_name(variable));

            let label = if readstat_variable_get_label(variable) != ptr::null() {
                ptr_to_str!(readstat_variable_get_label(variable))
            } else {
                String::new()
            };

            let type_ = readstat_variable_get_type(variable);

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
                type_: VariableType::from(type_),
                value_format: value_format,
                value_labels: value_labels,
            }
        }
    }
}

impl<'a> From<&'a str> for Variable {
    fn from(s: &str) -> Self {
        Variable {
            index: 0i32,
            name: s.to_string(),
            label: String::new(),
            type_: VariableType::Text,
            value_format: String::new(),
            value_labels: String::new(),
        }
    }
}

#[derive(Serialize, Clone, Hash, PartialEq, Eq)]
pub enum VariableType {
    Text,
    Numeric,
}

impl std::fmt::Debug for VariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VariableType::Text => write!(f, "String"),
            VariableType::Numeric => write!(f, "Numeric"),
        }
    }
}

impl From<readstat_type_t> for VariableType {
    fn from(t: readstat_type_t) -> Self {
        use self::readstat_type_t::{READSTAT_TYPE_STRING, READSTAT_TYPE_STRING_REF};

        match t {
            READSTAT_TYPE_STRING => VariableType::Text,
            READSTAT_TYPE_STRING_REF => VariableType::Text,
            _ => VariableType::Numeric,
        }
    }
}

