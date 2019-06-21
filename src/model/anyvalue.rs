use std::borrow::Cow;

use std::collections::hash_map::DefaultHasher;

use std::cmp::{Eq, PartialEq};
use std::fmt::{Display, Formatter, Result};
use std::hash::{Hash, Hasher};

use readstat::bindings::*;
use std::ffi::CStr;

/// AnyValue enum allows us to store each colum with different data type.
#[derive(Serialize, Debug, Clone)]
pub enum AnyValue {
    Str(Box<String>),
    Int8(Box<i8>),
    Int16(Box<i16>),
    Int32(Box<i32>),
    Float(Box<f32>),
    Double(Box<f64>),
}

impl Display for AnyValue {
    fn fmt(&self, f: &mut Formatter) -> Result {
        fn format_float<'a>(s: &'a str) -> Cow<'a, str> {
            if !s.contains('.') && !s.contains("NaN") {
                format!("{}.0", s).into()
            } else {
                s.into()
            }
        }

        use self::AnyValue::*;

        match self {
            &Str(ref value) => write!(f, "{}", *value),
            &Int8(ref value) => write!(f, "{}", *value),
            &Int16(ref value) => write!(f, "{}", *value),
            &Int32(ref value) => write!(f, "{}", *value),
            &Float(ref value) => write!(f, "{}", format_float(&format!("{:?}", *value))),
            &Double(ref value) => write!(f, "{}", format_float(&format!("{:?}", *value))),
        }
    }
}

impl From<readstat_value_t> for AnyValue {
    fn from(value: readstat_value_t) -> AnyValue {
        use self::readstat_type_t::*;
        use self::AnyValue::*;

        unsafe {
            match readstat_value_type(value) {
                READSTAT_TYPE_STRING => Str(Box::new(ptr_to_str!(readstat_string_value(value)))),
                READSTAT_TYPE_INT8 => Int8(Box::new(readstat_int8_value(value) as i8)),
                READSTAT_TYPE_INT16 => Int16(Box::new(readstat_int16_value(value))),
                READSTAT_TYPE_INT32 => Int32(Box::new(readstat_int32_value(value))),
                READSTAT_TYPE_FLOAT => Float(Box::new(readstat_float_value(value))),
                READSTAT_TYPE_DOUBLE => Double(Box::new(readstat_double_value(value))),
                READSTAT_TYPE_STRING_REF => Str(Box::new("REF TYPE".to_string())),
            }
        }
    }
}

impl<'a> From<&'a str> for AnyValue {
    fn from(s: &'a str) -> Self {
        use self::AnyValue::Str;

        Str(Box::new(Cow::Borrowed(s).into_owned()))
    }
}

impl From<i32> for AnyValue {
    fn from(i: i32) -> Self {
        use self::AnyValue::Int32;

        Int32(Box::new(Cow::Borrowed(&i).into_owned()))
    }
}

/// Hash trait allows for use a key in a HashMap
impl Hash for AnyValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{}", (*self)).hash(state);
    }
}

impl PartialEq for AnyValue {
    fn eq(&self, other: &AnyValue) -> bool {
        calculate_hash(self) == calculate_hash(other)
    }
}

impl Eq for AnyValue {}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    // use super::*;

}
