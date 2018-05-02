
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Report {
    pub metadata: Metadata,
    pub odd_characters: Option<Vec<HashMap<String, String>>>,
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

