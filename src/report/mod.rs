use check::CheckName;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::IntoIterator;
use std::slice::Iter;

use model::variable::VariableType;

pub mod html;

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

impl<'a> IntoIterator for &'a Report {
    type Item = (&'a CheckName, &'a Status);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut lst = vec![];
        for t in self.summary.iter() {
            lst.push(t);
        }
        lst.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        lst.into_iter()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Metadata {
    pub file_name: String,

    pub raw_case_count: i32,
    pub case_count: Option<i32>,
    pub variable_count: i32,
    pub data_type_occurences: HashMap<VariableType, i32>,

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
            data_type_occurences: HashMap::new(),

            creation_time: 0,
            modified_time: 0,

            file_label: "".into(),
            file_format_version: 0,
            file_encoding: None,

            compression: "".into(),
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum Category {
    BasicFile,
    Metadata,
    DataIntegrity,
    DisclosureRisk,
}

impl Category {
    fn variants() -> Iter<'static, Category> {
        use self::Category::*;
        static VARIANTS: &'static [Category] =
            &[BasicFile, Metadata, DataIntegrity, DisclosureRisk];
        VARIANTS.iter()
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::Category::*;

        match self {
            BasicFile => write!(f, "Basic File Checks"),
            Metadata => write!(f, "Metadata Checks"),
            DataIntegrity => write!(f, "Data Integrity Checks"),
            DisclosureRisk => write!(f, "Disclosure Risk Checks"),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Status {
    pub pass: i32,
    pub fail: i32,
    pub desc: String,
    pub locators: Option<HashSet<Locator>>,
    pub category: Category,
}

impl<'a> IntoIterator for &'a Status {
    type Item = &'a Locator;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut lst = vec![];
        if let Some(locators) = &self.locators {
            lst = locators.iter().collect::<Vec<Self::Item>>();
            lst.sort_by(|a, b| a.partial_cmp(b).unwrap());
            lst.into_iter()
        } else {
            lst.into_iter()
        }
    }
}

impl Status {
    pub fn new(desc: &str, category: Category) -> Status {
        Status {
            pass: 0,
            fail: 0,
            desc: desc.to_string(),
            locators: None,
            category,
        }
    }
}

#[derive(Serialize, Debug, Clone, Hash)]
pub struct Locator {
    pub variable_name: String,
    pub variable_index: i32,
    pub value_index: i32,
}

impl Ord for Locator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.variable_index.cmp(&other.variable_index)
    }
}

impl PartialOrd for Locator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Locator {
    fn eq(&self, other: &Self) -> bool {
        self.variable_index == other.variable_index
    }
}
impl Eq for Locator {}

impl Locator {
    pub fn new(
        variable_name: String,
        variable_index: i32,
        value_index: i32,
    ) -> Locator {
        Locator {
            variable_name: variable_name,
            variable_index: variable_index,
            value_index: value_index,
        }
    }
}
