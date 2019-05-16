use model::anyvalue::AnyValue;
use model::missing::Missing;

use model::variable::Variable;

use std::hash::{Hash, Hasher};

#[derive(Serialize, Debug, Clone)]
pub struct Value {
    pub variable: Variable,
    pub row: i32,
    pub value: AnyValue,
    pub label: String,
    pub missing: Missing,
}

impl<'a> From<&'a str> for Value {
    fn from(s: &str) -> Self {
        Value {
            variable: Variable::from("foo"),
            row: 0,
            value: AnyValue::from(s),
            label: String::new(),
            missing: Missing::NOT_MISSING,
        }
    }
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
