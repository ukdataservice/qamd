
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
    Spellcheck,
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

pub fn only_contains(string: &str, patterns: &Vec<String>) -> bool {
    string.split(" ")
        .map(|w| patterns.contains(&w.to_string()))
        .fold(true, |a, b| a && b)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let patterns = vec!["bar".to_string()];

        assert!(contains("foo bar baz", &patterns));
        assert_eq!(contains("foo baz qux", &patterns), false);
    }

    #[test]
    fn test_only_contains() {
        let patterns = vec!["foo", "baz", "qux"].iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(only_contains("foo baz qux", &patterns));
        assert_eq!(only_contains("foo bar baz", &patterns), false);
    }

    #[test]
    fn test_to_sentence() {
        assert_eq!(to_sentence("ThisIsASentence"), "This is a sentence");
        assert_eq!(to_sentence("thisIsAlsoASentence"), "This is also a sentence");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("word"), "Word".to_string());
    }
}

