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

    /// 检查两个 Oracle 类型是否兼容。
    ///
    /// 字符类型（CHAR/NCHAR/VARCHAR2/NVARCHAR2/CLOB/NCLOB）间互兼容；
    /// 数字类型（NUMBER/BINARY_FLOAT/BINARY_DOUBLE）间互兼容；
    /// 时间类型（DATE/TIMESTAMP/TIMESTAMP WITH TIME ZONE/LOCAL TZ）间互兼容。
    fn type_compatible(&self, other: &Self) -> bool {
        use OracleTypeInfo::*;

        if self == other {
            return true;
        }

        /// 是否为字符/布尔类类型。
        fn is_string(t: &OracleTypeInfo) -> bool {
            matches!(t, Char | NChar | Varchar2 | NVarchar2 | Clob | NClob | Long | Boolean)
        }
        /// 是否为数字类类型。
        fn is_number(t: &OracleTypeInfo) -> bool {
            matches!(t, Number | BinaryFloat | BinaryDouble)
        }
        /// 是否为时间类类型。
        fn is_datetime(t: &OracleTypeInfo) -> bool {
            matches!(t, Date | Timestamp | TimestampTZ | TimestampLTZ)
        }
        /// 是否为 RAW/BLOB 二进制类型。
        fn is_binary(t: &OracleTypeInfo) -> bool {
            matches!(t, Raw | LongRaw | Blob)
        }

        is_string(self) && is_string(other)
            || is_number(self) && is_number(other)
            || is_datetime(self) && is_datetime(other)
            || is_binary(self) && is_binary(other)
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

    #[test]
    fn test_type_compatible_string_group() {
        let strings = [OracleTypeInfo::Char, OracleTypeInfo::NChar, OracleTypeInfo::Varchar2,
                       OracleTypeInfo::NVarchar2, OracleTypeInfo::Clob, OracleTypeInfo::NClob,
                       OracleTypeInfo::Long, OracleTypeInfo::Boolean];
        for a in &strings {
            for b in &strings {
                assert!(a.type_compatible(b), "{a:?} should be compatible with {b:?}");
            }
        }
    }

    #[test]
    fn test_type_compatible_number_group() {
        let numbers = [OracleTypeInfo::Number, OracleTypeInfo::BinaryFloat, OracleTypeInfo::BinaryDouble];
        for a in &numbers {
            for b in &numbers {
                assert!(a.type_compatible(b), "{a:?} should be compatible with {b:?}");
            }
        }
    }

    #[test]
    fn test_type_compatible_datetime_group() {
        let dts = [OracleTypeInfo::Date, OracleTypeInfo::Timestamp,
                   OracleTypeInfo::TimestampTZ, OracleTypeInfo::TimestampLTZ];
        for a in &dts {
            for b in &dts {
                assert!(a.type_compatible(b), "{a:?} should be compatible with {b:?}");
            }
        }
    }

    #[test]
    fn test_type_compatible_binary_group() {
        let binaries = [OracleTypeInfo::Raw, OracleTypeInfo::LongRaw, OracleTypeInfo::Blob];
        for a in &binaries {
            for b in &binaries {
                assert!(a.type_compatible(b), "{a:?} should be compatible with {b:?}");
            }
        }
    }

    #[test]
    fn test_type_compatible_cross_group_incompatible() {
        let string_t = OracleTypeInfo::Varchar2;
        let number_t = OracleTypeInfo::Number;
        let datetime_t = OracleTypeInfo::Date;
        let binary_t = OracleTypeInfo::Raw;

        assert!(!string_t.type_compatible(&number_t));
        assert!(!string_t.type_compatible(&datetime_t));
        assert!(!string_t.type_compatible(&binary_t));
        assert!(!number_t.type_compatible(&datetime_t));
        assert!(!number_t.type_compatible(&binary_t));
        assert!(!datetime_t.type_compatible(&binary_t));
    }

    #[test]
    fn test_type_compatible_same_variant() {
        assert!(OracleTypeInfo::Null.type_compatible(&OracleTypeInfo::Null));
        assert!(OracleTypeInfo::RowID.type_compatible(&OracleTypeInfo::RowID));
        assert!(OracleTypeInfo::Unknown("A".into()).type_compatible(&OracleTypeInfo::Unknown("A".into())));
    }

    #[test]
    fn test_type_compatible_unspecified_not_cross_group() {
        // IntervalYM, IntervalDS, RowID, Unknown, Null are not in any group
        let standalone = [
            OracleTypeInfo::IntervalYM,
            OracleTypeInfo::IntervalDS,
            OracleTypeInfo::RowID,
            OracleTypeInfo::Null,
        ];
        let other = OracleTypeInfo::Varchar2;
        for s in &standalone {
            assert!(!s.type_compatible(&other), "{s:?} should not be compatible with {other:?}");
            assert!(!other.type_compatible(s), "{other:?} should not be compatible with {s:?}");
        }
    }

    #[test]
    fn test_display_all_variants() {
        assert_eq!(format!("{}", OracleTypeInfo::Char), "CHAR");
        assert_eq!(format!("{}", OracleTypeInfo::NChar), "NCHAR");
        assert_eq!(format!("{}", OracleTypeInfo::Varchar2), "VARCHAR2");
        assert_eq!(format!("{}", OracleTypeInfo::NVarchar2), "NVARCHAR2");
        assert_eq!(format!("{}", OracleTypeInfo::Clob), "CLOB");
        assert_eq!(format!("{}", OracleTypeInfo::NClob), "NCLOB");
        assert_eq!(format!("{}", OracleTypeInfo::Long), "LONG");
        assert_eq!(format!("{}", OracleTypeInfo::Raw), "RAW");
        assert_eq!(format!("{}", OracleTypeInfo::LongRaw), "LONG RAW");
        assert_eq!(format!("{}", OracleTypeInfo::Blob), "BLOB");
        assert_eq!(format!("{}", OracleTypeInfo::Number), "NUMBER");
        assert_eq!(format!("{}", OracleTypeInfo::BinaryFloat), "BINARY_FLOAT");
        assert_eq!(format!("{}", OracleTypeInfo::BinaryDouble), "BINARY_DOUBLE");
        assert_eq!(format!("{}", OracleTypeInfo::Date), "DATE");
        assert_eq!(format!("{}", OracleTypeInfo::Timestamp), "TIMESTAMP");
        assert_eq!(format!("{}", OracleTypeInfo::TimestampTZ), "TIMESTAMP WITH TIME ZONE");
        assert_eq!(format!("{}", OracleTypeInfo::TimestampLTZ), "TIMESTAMP WITH LOCAL TIME ZONE");
        assert_eq!(format!("{}", OracleTypeInfo::IntervalYM), "INTERVAL YEAR TO MONTH");
        assert_eq!(format!("{}", OracleTypeInfo::IntervalDS), "INTERVAL DAY TO SECOND");
        assert_eq!(format!("{}", OracleTypeInfo::RowID), "ROWID");
        assert_eq!(format!("{}", OracleTypeInfo::Boolean), "BOOLEAN");
        assert_eq!(format!("{}", OracleTypeInfo::Null), "NULL");
    }

    #[test]
    fn test_type_info_is_void_always_false() {
        assert!(!OracleTypeInfo::Char.is_void());
        assert!(!OracleTypeInfo::Number.is_void());
        assert!(!OracleTypeInfo::Null.is_void());
    }

    #[test]
    fn test_type_info_name_matches_display() {
        let variants = [
            OracleTypeInfo::Char,
            OracleTypeInfo::Number,
            OracleTypeInfo::BinaryFloat,
            OracleTypeInfo::Date,
            OracleTypeInfo::Timestamp,
            OracleTypeInfo::TimestampTZ,
            OracleTypeInfo::TimestampLTZ,
            OracleTypeInfo::Blob,
            OracleTypeInfo::Boolean,
            OracleTypeInfo::Null,
            OracleTypeInfo::RowID,
            OracleTypeInfo::IntervalYM,
            OracleTypeInfo::IntervalDS,
        ];
        for v in &variants {
            assert_eq!(v.name(), &format!("{v}"), "name() should equal Display for {v:?}");
        }
    }
}
