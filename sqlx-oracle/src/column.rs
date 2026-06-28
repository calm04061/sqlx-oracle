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

    /// 返回列在结果集中的序号（从 0 开始）。
    fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// 返回列名（来自 OCI 元数据）。
    fn name(&self) -> &str {
        &self.name
    }

    /// 返回 Oracle 数据类型信息。
    fn type_info(&self) -> &OracleTypeInfo {
        &self.type_info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::column::Column;

    // -----------------------------------------------------------------------
    // 基本构造和 trait 方法
    // -----------------------------------------------------------------------

    #[test]
    fn test_column_construction() {
        let col = OracleColumn {
            ordinal: 0,
            name: "ID".to_string(),
            type_info: OracleTypeInfo::Number,
        };
        assert_eq!(col.ordinal(), 0);
        assert_eq!(col.name(), "ID");
        assert_eq!(col.type_info(), &OracleTypeInfo::Number);
    }

    #[test]
    fn test_column_ordinal() {
        let col = OracleColumn {
            ordinal: 5,
            name: "NAME".to_string(),
            type_info: OracleTypeInfo::Varchar2,
        };
        assert_eq!(col.ordinal(), 5);
    }

    // -----------------------------------------------------------------------
    // 列名格式
    // -----------------------------------------------------------------------

    #[test]
    fn test_column_name_various() {
        let col = OracleColumn {
            ordinal: 0,
            name: "column_name_with_underscores".to_string(),
            type_info: OracleTypeInfo::Varchar2,
        };
        assert_eq!(col.name(), "column_name_with_underscores");

        let col = OracleColumn {
            ordinal: 1,
            name: "SYS_C0012345".to_string(),
            type_info: OracleTypeInfo::Number,
        };
        assert_eq!(col.name(), "SYS_C0012345");
    }

    // -----------------------------------------------------------------------
    // 类型映射
    // -----------------------------------------------------------------------

    #[test]
    fn test_column_type_info_different_types() {
        let types = [
            OracleTypeInfo::Varchar2,
            OracleTypeInfo::Number,
            OracleTypeInfo::Date,
            OracleTypeInfo::Blob,
            OracleTypeInfo::TimestampTZ,
            OracleTypeInfo::Null,
        ];
        for (i, t) in types.iter().enumerate() {
            let col = OracleColumn {
                ordinal: i,
                name: format!("COL_{i}"),
                type_info: t.clone(),
            };
            assert_eq!(col.type_info(), t);
        }
    }

    // -----------------------------------------------------------------------
    // Debug / Clone
    // -----------------------------------------------------------------------

    #[test]
    fn test_column_debug() {
        let col = OracleColumn {
            ordinal: 0,
            name: "X".to_string(),
            type_info: OracleTypeInfo::Number,
        };
        let debug = format!("{col:?}");
        assert!(debug.contains("X"));
        assert!(debug.contains("Number"));
    }

    #[test]
    fn test_column_clone() {
        let a = OracleColumn {
            ordinal: 1,
            name: "A".to_string(),
            type_info: OracleTypeInfo::Varchar2,
        };
        let b = a.clone();
        assert_eq!(a.ordinal(), b.ordinal());
        assert_eq!(a.name(), b.name());
        assert_eq!(a.type_info(), b.type_info());
    }
}
