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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::column::{Column, ColumnIndex};
    use sqlx_core::row::Row;
    use crate::OracleTypeInfo;

    fn make_row() -> OracleRow {
        OracleRow {
            columns: vec![
                OracleColumn { ordinal: 0, name: "ID".to_string(), type_info: OracleTypeInfo::Number },
                OracleColumn { ordinal: 1, name: "NAME".to_string(), type_info: OracleTypeInfo::Varchar2 },
                OracleColumn { ordinal: 2, name: "CREATED_AT".to_string(), type_info: OracleTypeInfo::Date },
            ],
            values: vec![
                OracleValue { value: Some(b"1".to_vec()), type_info: OracleTypeInfo::Number },
                OracleValue { value: Some(b"Alice".to_vec()), type_info: OracleTypeInfo::Varchar2 },
                OracleValue { value: Some(b"2024-01-15 10:30:45".to_vec()), type_info: OracleTypeInfo::Date },
            ],
        }
    }

    #[test]
    fn test_row_columns() {
        let row = make_row();
        let cols = row.columns();
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0].name(), "ID");
        assert_eq!(cols[1].name(), "NAME");
        assert_eq!(cols[2].name(), "CREATED_AT");
    }

    #[test]
    fn test_row_get_raw_by_index() {
        let row = make_row();
        let val = row.try_get_raw(0).unwrap();
        assert_eq!(val.value, Some(b"1" as &[u8]));

        let val = row.try_get_raw(1).unwrap();
        assert_eq!(val.value, Some(b"Alice" as &[u8]));
    }

    #[test]
    fn test_row_get_raw_by_name_case_sensitive() {
        let row = make_row();
        let val = row.try_get_raw("ID").unwrap();
        assert_eq!(val.value, Some(b"1" as &[u8]));
    }

    #[test]
    fn test_row_get_raw_by_name_case_insensitive() {
        let row = make_row();
        let val = row.try_get_raw("id").unwrap();
        assert_eq!(val.value, Some(b"1" as &[u8]));

        let val = row.try_get_raw("Id").unwrap();
        assert_eq!(val.value, Some(b"1" as &[u8]));
    }

    #[test]
    fn test_row_get_raw_by_name_partial() {
        let row = make_row();
        let val = row.try_get_raw("NAME").unwrap();
        assert_eq!(val.value, Some(b"Alice" as &[u8]));
    }

    #[test]
    fn test_row_get_raw_index_out_of_bounds() {
        let row = make_row();
        let result = row.try_get_raw(5usize);
        assert!(result.is_err());
    }

    #[test]
    fn test_row_get_raw_name_not_found() {
        let row = make_row();
        let result = row.try_get_raw("NONEXISTENT");
        assert!(result.is_err());
    }

    #[test]
    fn test_row_debug() {
        let row = make_row();
        let debug = format!("{row:?}");
        assert!(debug.contains("ID"));
        assert!(debug.contains("columns"));
        assert!(debug.contains("values"));
    }

    #[test]
    fn test_row_clone() {
        let row = make_row();
        let cloned = row.clone();
        assert_eq!(cloned.columns.len(), row.columns.len());
        assert_eq!(cloned.values.len(), row.values.len());
    }

    #[test]
    fn test_column_index_usize() {
        let row = make_row();
        assert_eq!(0usize.index(&row).unwrap(), 0);
        assert_eq!(1usize.index(&row).unwrap(), 1);
        assert!(5usize.index(&row).is_err());
    }

    #[test]
    fn test_column_index_str() {
        let row = make_row();
        assert_eq!("ID".index(&row).unwrap(), 0);
        assert_eq!("id".index(&row).unwrap(), 0);
        assert_eq!("NAME".index(&row).unwrap(), 1);
        assert_eq!("name".index(&row).unwrap(), 1);
        assert_eq!("CREATED_AT".index(&row).unwrap(), 2);
        assert!("NONEXISTENT".index(&row).is_err());
    }

    #[test]
    fn test_row_empty() {
        let row = OracleRow {
            columns: vec![],
            values: vec![],
        };
        assert_eq!(row.columns().len(), 0);
        assert!(row.try_get_raw(0usize).is_err());
    }
}
