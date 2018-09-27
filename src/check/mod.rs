
use readstat::context::Context;
use config::Config;
use model::value::Value;
use model::variable::Variable;
use report::Report;

use std::fmt;

type CheckFn<T> = fn(value: &T,
                     config: &Config,
                     report: &mut Report);

pub type VariableCheckFn = CheckFn<Variable>;
pub type ValueCheckFn = CheckFn<Value>;
pub type PostCheckFn = fn(context: &mut Context);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum CheckName {
    DateFormat,
    MissingVariableLabels,
    VariableLabelMaxLength,
    VariableOddCharacters,

    ValueDefinedMissingNoLabel,

    SystemMissingOverThreshold,
    VariablesWithUniqueValues,
    ValueLabelMaxLength,
    ValueOddCharacters,
    ValueRegexPatterns,
}

impl fmt::Display for CheckName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = format!("{:?}", self);

        write!(f, "{}", to_sentence(&name))
    }
}

/// Holds lists of checks to be run
pub struct Check {
    pub variable: Vec<VariableCheckFn>,
    pub value: Vec<ValueCheckFn>,
    pub post: Vec<PostCheckFn>,
}

impl Check {
    pub fn new() -> Check {
        Check {
            variable: variable::register(),
            value: value::register(),
            post: post::register(),
        }
    }
}

impl fmt::Debug for Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{{ variable: {}, value: {} }}",
               self.variable.len(),
               self.value.len())
    }
}

pub fn contains(string: &str, patterns: &Vec<String>) -> bool {
    patterns.iter()
        .map(|p| string.contains(p))
        .fold(false, |a, b| { a || b })
}

fn to_sentence(s: &str) -> String {
    let r = s.chars().fold(String::new(), |a, b| {
        if b.is_uppercase() {
            format!("{} {}", a, b)
        } else {
            format!("{}{}", a, b)
        }
    })
    .trim()
    .to_lowercase();

    capitalize(&r)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[macro_use]
mod macros;
pub mod variable;
pub mod value;
pub mod post;

