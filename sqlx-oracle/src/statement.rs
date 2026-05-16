use std::borrow::Cow;

use crate::OracleColumn;
use crate::Oracle;
use crate::OracleArguments;
use crate::OracleTypeInfo;
use sqlx_core::statement::Statement;
use sqlx_core::Either;

#[derive(Debug, Clone)]
pub struct OracleStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) columns: Vec<OracleColumn>,
}

impl<'q> Statement<'q> for OracleStatement<'q> {
    type Database = Oracle;

    fn to_owned(&self) -> OracleStatement<'static> {
        OracleStatement {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            columns: self.columns.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[OracleTypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[OracleColumn] {
        &self.columns
    }

    impl_statement_query!(OracleArguments);
}

impl sqlx_core::column::ColumnIndex<OracleStatement<'_>> for usize {
    fn index(&self, statement: &OracleStatement<'_>) -> Result<usize, sqlx_core::error::Error> {
        let len = statement.columns.len();
        if *self >= len {
            return Err(sqlx_core::error::Error::ColumnIndexOutOfBounds { len, index: *self });
        }
        Ok(*self)
    }
}

impl sqlx_core::column::ColumnIndex<OracleStatement<'_>> for &'_ str {
    fn index(&self, statement: &OracleStatement<'_>) -> Result<usize, sqlx_core::error::Error> {
        statement
            .columns
            .iter()
            .position(|col| col.name.eq_ignore_ascii_case(self))
            .ok_or_else(|| sqlx_core::error::Error::ColumnNotFound((*self).into()))
    }
}
