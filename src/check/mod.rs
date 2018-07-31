
#[macro_use]
mod macros;
pub mod variable;
pub mod value;
pub mod post;

use readstat::context::Context;
use config::Config;
use report::{Report, Variable, Value};

use std::fmt;

type CheckFn<T> = fn(value: &T,
                     config: &Config,
                     report: &mut Report);

pub type VariableCheckFn = CheckFn<Variable>;
pub type ValueCheckFn = CheckFn<Value>;
pub type PostCheckFn = CheckFn<Context>;

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
        .fold(false, |a, b| a || b)
}

