use crate::OracleColumn;
use crate::Oracle;
use crate::OracleArguments;
use crate::OracleTypeInfo;
use sqlx_core::sql_str::SqlStr;
use sqlx_core::statement::Statement;
use sqlx_core::Either;

/// 预编译语句。
///
/// 保存转换后的 Oracle SQL（`?` → `:n` 格式）以及查询结果列元数据。
#[derive(Debug, Clone)]
pub struct OracleStatement {
    pub(crate) sql: SqlStr,
    pub(crate) columns: Vec<OracleColumn>,
}

impl Statement for OracleStatement {
    type Database = Oracle;

    fn into_sql(self) -> SqlStr {
        self.sql
    }

    fn sql(&self) -> &SqlStr {
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

impl sqlx_core::column::ColumnIndex<OracleStatement> for usize {
    fn index(&self, statement: &OracleStatement) -> Result<usize, sqlx_core::error::Error> {
        let len = statement.columns.len();
        if *self >= len {
            return Err(sqlx_core::error::Error::ColumnIndexOutOfBounds { len, index: *self });
        }
        Ok(*self)
    }
}

impl sqlx_core::column::ColumnIndex<OracleStatement> for &'_ str {
    fn index(&self, statement: &OracleStatement) -> Result<usize, sqlx_core::error::Error> {
        statement
            .columns
            .iter()
            .position(|col| col.name.eq_ignore_ascii_case(self))
            .ok_or_else(|| sqlx_core::error::Error::ColumnNotFound((*self).into()))
    }
}
