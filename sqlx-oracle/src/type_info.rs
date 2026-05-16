use std::fmt::{Debug, Display, Formatter};
use sqlx_core::type_info::TypeInfo;

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
