use std::fmt::{Debug, Display, Formatter};
use sqlx_core::type_info::TypeInfo;

/// Oracle 数据类型枚举。
///
/// 覆盖常用 Oracle 数据类型，映射自 sibyl 的 `ColumnType`。
#[derive(Debug, Clone, PartialEq)]
pub enum OracleTypeInfo {
    Char,
    NChar,
    Varchar2,
    NVarchar2,
    Clob,
    NClob,
    Long,
    Raw,
    LongRaw,
    Blob,
    Number,
    BinaryFloat,
    BinaryDouble,
    Date,
    Timestamp,
    TimestampTZ,
    TimestampLTZ,
    IntervalYM,
    IntervalDS,
    RowID,
    Boolean,
    Unknown(String),
    Null,
}

impl OracleTypeInfo {
    /// 返回 Oracle 类型名称（用于 Display 和调试输出）。
    pub fn name(&self) -> &str {
        match self {
            OracleTypeInfo::Char => "CHAR",
            OracleTypeInfo::NChar => "NCHAR",
            OracleTypeInfo::Varchar2 => "VARCHAR2",
            OracleTypeInfo::NVarchar2 => "NVARCHAR2",
            OracleTypeInfo::Clob => "CLOB",
            OracleTypeInfo::NClob => "NCLOB",
            OracleTypeInfo::Long => "LONG",
            OracleTypeInfo::Raw => "RAW",
            OracleTypeInfo::LongRaw => "LONG RAW",
            OracleTypeInfo::Blob => "BLOB",
            OracleTypeInfo::Number => "NUMBER",
            OracleTypeInfo::BinaryFloat => "BINARY_FLOAT",
            OracleTypeInfo::BinaryDouble => "BINARY_DOUBLE",
            OracleTypeInfo::Date => "DATE",
            OracleTypeInfo::Timestamp => "TIMESTAMP",
            OracleTypeInfo::TimestampTZ => "TIMESTAMP WITH TIME ZONE",
            OracleTypeInfo::TimestampLTZ => "TIMESTAMP WITH LOCAL TIME ZONE",
            OracleTypeInfo::IntervalYM => "INTERVAL YEAR TO MONTH",
            OracleTypeInfo::IntervalDS => "INTERVAL DAY TO SECOND",
            OracleTypeInfo::RowID => "ROWID",
            OracleTypeInfo::Boolean => "BOOLEAN",
            OracleTypeInfo::Unknown(s) => s,
            OracleTypeInfo::Null => "NULL",
        }
    }
}

impl TypeInfo for OracleTypeInfo {
    fn is_null(&self) -> bool {
        matches!(self, OracleTypeInfo::Null)
    }

    fn name(&self) -> &str {
        self.name()
    }

    fn is_void(&self) -> bool {
        false
    }
}

impl Display for OracleTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::type_info::TypeInfo;

    #[test]
    fn test_names() {
        assert_eq!(OracleTypeInfo::Char.name(), "CHAR");
        assert_eq!(OracleTypeInfo::Varchar2.name(), "VARCHAR2");
        assert_eq!(OracleTypeInfo::Number.name(), "NUMBER");
        assert_eq!(OracleTypeInfo::BinaryFloat.name(), "BINARY_FLOAT");
        assert_eq!(OracleTypeInfo::Date.name(), "DATE");
        assert_eq!(OracleTypeInfo::TimestampTZ.name(), "TIMESTAMP WITH TIME ZONE");
        assert_eq!(OracleTypeInfo::Boolean.name(), "BOOLEAN");
        assert_eq!(OracleTypeInfo::Null.name(), "NULL");
        assert_eq!(OracleTypeInfo::Unknown("MYTYPE".into()).name(), "MYTYPE");
    }

    #[test]
    fn test_is_null() {
        assert!(OracleTypeInfo::Null.is_null());
        assert!(!OracleTypeInfo::Char.is_null());
        assert!(!OracleTypeInfo::Number.is_null());
    }

    #[test]
    fn test_display_matches_name() {
        let t = OracleTypeInfo::Varchar2;
        assert_eq!(format!("{t}"), t.name());
    }

    #[test]
    fn test_eq() {
        assert_eq!(OracleTypeInfo::Char, OracleTypeInfo::Char);
        assert_ne!(OracleTypeInfo::Char, OracleTypeInfo::Number);
        assert_eq!(
            OracleTypeInfo::Unknown("A".into()),
            OracleTypeInfo::Unknown("A".into())
        );
    }

    #[test]
    fn test_clone() {
        let a = OracleTypeInfo::TimestampTZ;
        let b = a.clone();
        assert_eq!(a, b);
    }
}
