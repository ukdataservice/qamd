
pub mod variable;
pub mod value;
pub mod common;

use report::{Variable, Value};

use std::os::raw::c_void;
use std::fmt;

pub type VariableCheckFn = fn(variable: &Variable, ctx: *mut c_void);
pub type ValueCheckFn = fn(value: &Value, ctx: *mut c_void);

pub struct Check {
    pub variable: Vec<VariableCheckFn>,
    pub value: Vec<ValueCheckFn>,
}

impl Check {
    pub fn new() -> Check {
        Check {
            variable: variable::register(),
            value: value::register(),
        }
    }
}

impl fmt::Debug for Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ variable: {}, value: {} }}", self.variable.len(), self.value.len())
    }
}


