use crate::OracleTypeInfo;
use crate::Oracle;
use sqlx_core::column::Column;

#[derive(Debug, Clone)]
pub struct OracleColumn {
    pub ordinal: usize,
    pub name: String,
    pub type_info: OracleTypeInfo,
}

impl Column for OracleColumn {
    type Database = Oracle;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &OracleTypeInfo {
        &self.type_info
    }
}
