
// use std::collections::hash_map::DefaultHasher;

use std::borrow::Cow;

use std::fmt::{Display, Formatter, Result};
// use std::hash::{Hash, Hasher};
// use std::cmp::{PartialEq, Eq};

use std::ffi::CStr;
use bindings::*;

/// AnyValue enum allows us to store each colum with different data type.
#[derive(Serialize, Debug, Clone)]
pub enum AnyValue {
    Str(Box<String>),
    Int8(Box<i8>),
    Int16(Box<i16>),
    Int32(Box<i32>),
    Float(Box<f32>),
    Double(Box<f64>)
}

// impl Hash for AnyValue {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         format!("{}", (*self)).hash(state);
//     }
// }

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
            &Float(ref value) =>
                write!(f, "{}", format_float(&format!("{:?}", *value))),
            &Double(ref value) =>
                write!(f, "{}", format_float(&format!("{:?}", *value)))
        }
    }
}

// impl PartialEq for AnyValue {
//     fn eq(&self, other: &AnyValue) -> bool {
//         calculate_hash(self) == calculate_hash(other)
//     }
// }
// 
// impl Eq for AnyValue {}

impl From<readstat_value_t> for AnyValue {
    fn from(value: readstat_value_t) -> AnyValue {
        use readstat_type_t::*;
        use self::AnyValue::*;

        unsafe {
            match readstat_value_type(value) {
                READSTAT_TYPE_STRING =>
                    Str(Box::new(ptr_to_str!(readstat_string_value(value)))),
                READSTAT_TYPE_INT8 =>
                    Int8(Box::new(readstat_int8_value(value) as i8)),
                READSTAT_TYPE_INT16 =>
                    Int16(Box::new(readstat_int16_value(value))),
                READSTAT_TYPE_INT32 =>
                    Int32(Box::new(readstat_int32_value(value))),
                READSTAT_TYPE_FLOAT =>
                    Float(Box::new(readstat_float_value(value))),
                READSTAT_TYPE_DOUBLE =>
                    Double(Box::new(readstat_double_value(value))),
                READSTAT_TYPE_STRING_REF =>
                    Str(Box::new("REF TYPE".to_string())),
            }
        }
    }
}

// pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
//     let mut s = DefaultHasher::new();
//     t.hash(&mut s);
//     s.finish()
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_value_hash_str() {
    //     let any1 = AnyValue::Str(Box::new("foo".into()));
    //     let any2 = AnyValue::Str(Box::new("foo".into()));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_int8() {
    //     let any1 = AnyValue::Int8(Box::new(5));
    //     let any2 = AnyValue::Int8(Box::new(5));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_int16() {
    //     let any1 = AnyValue::Int16(Box::new(5));
    //     let any2 = AnyValue::Int16(Box::new(5));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_int32() {
    //     let any1 = AnyValue::Int32(Box::new(5));
    //     let any2 = AnyValue::Int32(Box::new(5));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_float() {
    //     let any1 = AnyValue::Float(Box::new(5.2));
    //     let any2 = AnyValue::Float(Box::new(5.2));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_double() {
    //     let any1 = AnyValue::Double(Box::new(5.2));
    //     let any2 = AnyValue::Double(Box::new(5.2));

    //     assert!(calculate_hash(&any1) == calculate_hash(&any2));
    // }

    // #[test]
    // fn test_value_hash_int32_double() {
    //     let any1 = AnyValue::Int32(Box::new(5));
    //     let any2 = AnyValue::Double(Box::new(5.0));

    //     let any3 = AnyValue::Float(Box::new(5.2));
    //     let any4 = AnyValue::Double(Box::new(5.0));

    //     assert!(calculate_hash(&any1) != calculate_hash(&any2));
    //     assert!(calculate_hash(&any3) != calculate_hash(&any4));
    // }
}

