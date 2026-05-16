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
