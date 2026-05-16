use crate::OracleColumn;
use crate::OracleValue;
use crate::OracleValueRef;
use crate::Oracle;
use sqlx_core::row::Row;
use sqlx_core::column::ColumnIndex;
use sqlx_core::error::Error;
use sqlx_core::value::Value;

/// Oracle 查询结果行。
///
/// 包含列元数据和列值，支持按序号和按名称（大小写不敏感）访问。
#[derive(Debug, Clone)]
pub struct OracleRow {
    pub(crate) columns: Vec<OracleColumn>,
    pub(crate) values: Vec<OracleValue>,
}

impl Row for OracleRow {
    type Database = Oracle;

    fn columns(&self) -> &[OracleColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<OracleValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = &self.values[index];
        Ok(value.as_ref())
    }
}

impl ColumnIndex<OracleRow> for usize {
    fn index(&self, row: &OracleRow) -> Result<usize, Error> {
        let len = row.columns.len();
        if *self >= len {
            return Err(Error::ColumnIndexOutOfBounds { len, index: *self });
        }
        Ok(*self)
    }
}

impl ColumnIndex<OracleRow> for &'_ str {
    fn index(&self, row: &OracleRow) -> Result<usize, Error> {
        row.columns
            .iter()
            .position(|col| col.name.eq_ignore_ascii_case(self))
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}
