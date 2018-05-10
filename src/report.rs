
// use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub variable_checks: VariableChecks,
    pub value_checks: ValueChecks,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariableChecks {
    pub odd_characters: Option<Vec<Variable>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueChecks {
    pub odd_characters: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variable {
    pub index: i32,
    pub name: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub index: i32,
    pub row: i32,
    pub value: String,
    pub label: String,
}

