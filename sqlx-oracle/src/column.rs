use crate::OracleTypeInfo;
use crate::Oracle;
use sqlx_core::column::Column;

/// Oracle 列元数据。
///
/// 包含列序号、列名（来自 OCI）和 Oracle 类型信息。
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
