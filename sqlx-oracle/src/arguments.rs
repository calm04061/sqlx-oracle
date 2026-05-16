use std::fmt::Write;

use sqlx_core::arguments::Arguments;
use sqlx_core::encode::Encode;
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

use crate::Oracle;

#[derive(Debug, Clone)]
pub enum OracleBindValue {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Default, Clone)]
pub struct OracleArgumentBuffer {
    pub(crate) values: Vec<OracleBindValue>,
}

impl OracleArgumentBuffer {
    pub fn push(&mut self, value: OracleBindValue) {
        self.values.push(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Debug, Default, Clone)]
pub struct OracleArguments {
    pub(crate) buffer: OracleArgumentBuffer,
}

impl<'q> Arguments<'q> for OracleArguments {
    type Database = Oracle;

    fn reserve(&mut self, additional: usize, _size: usize) {
        self.buffer.values.reserve(additional);
    }

    fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: 'q + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let _ = Encode::encode(value, &mut self.buffer)?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.buffer.values.len()
    }

    fn format_placeholder<W: Write>(&self, writer: &mut W) -> std::fmt::Result {
        write!(writer, "${}", self.buffer.values.len())
    }
}

sqlx_core::impl_into_arguments_for_arguments!(OracleArguments);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_value_null() {
        let v = OracleBindValue::Null;
        match v {
            OracleBindValue::Null => (),
            _ => panic!("expected Null"),
        }
    }

    #[test]
    fn test_bind_value_int() {
        let v = OracleBindValue::Int(42);
        match v {
            OracleBindValue::Int(i) => assert_eq!(i, 42),
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn test_bind_value_float() {
        let v = OracleBindValue::Float(3.14);
        match v {
            OracleBindValue::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            _ => panic!("expected Float"),
        }
    }

    #[test]
    fn test_bind_value_string() {
        let v = OracleBindValue::String("hello".into());
        match v {
            OracleBindValue::String(s) => assert_eq!(s, "hello"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_bind_value_bool() {
        let v = OracleBindValue::Bool(true);
        match v {
            OracleBindValue::Bool(b) => assert!(b),
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn test_buffer_push_and_len() {
        let mut buf = OracleArgumentBuffer::default();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        buf.push(OracleBindValue::Int(1));
        assert_eq!(buf.len(), 1);
        assert!(!buf.is_empty());

        buf.push(OracleBindValue::String("x".into()));
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_arguments_default() {
        let args = OracleArguments::default();
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn test_arguments_add_i64() {
        let mut args = OracleArguments::default();
        args.add(42i64).unwrap();
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_arguments_add_string() {
        let mut args = OracleArguments::default();
        args.add("hello").unwrap();
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn test_arguments_add_multiple_types() {
        let mut args = OracleArguments::default();
        args.add(1i64).unwrap();
        args.add(3.14f64).unwrap();
        args.add(true).unwrap();
        args.add("text").unwrap();
        assert_eq!(args.len(), 4);
    }

    #[test]
    fn test_arguments_reserve() {
        let mut args = OracleArguments::default();
        args.reserve(10, 0);
        assert_eq!(args.buffer.values.capacity(), 10);
    }
}
