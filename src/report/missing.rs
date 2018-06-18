
#![allow(non_camel_case_types)]

use std::fmt::{Display, Formatter, Result};

/// Missing, represent the missingness of a variable
#[derive(Serialize, Hash, PartialEq, Eq, Debug, Clone)]
pub enum Missing {
    NOT_MISSING,
    SYSTEM_MISSING,
    TAGGED_MISSING(char),
    DEFINED_MISSING,
}

impl Display for Missing {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Missing::*;

        let s = match self {
            &NOT_MISSING => "NOT_MISSING".into(),
            &SYSTEM_MISSING => "SYSTEM_MISSING".into(),
            &TAGGED_MISSING(ref c) => format!("TAGGED_MISSING({})", c),
            &DEFINED_MISSING => "DEFINED_MISSING".into(),
        };

        write!(f, "{}", s)
    }
}

