use std::borrow::Cow;
use std::fmt::Debug;

use crate::OracleTypeInfo;
use crate::Oracle;
use sqlx_core::value::{Value, ValueRef};

#[derive(Debug, Clone)]
pub struct OracleValue {
    pub value: Option<Vec<u8>>,
    pub type_info: OracleTypeInfo,
}

#[derive(Clone)]
pub struct OracleValueRef<'r> {
    pub value: Option<&'r [u8]>,
    pub type_info: OracleTypeInfo,
}

impl<'r> Debug for OracleValueRef<'r> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OracleValueRef")
            .field("value", &self.value)
            .field("type_info", &self.type_info)
            .finish()
    }
}

impl Value for OracleValue {
    type Database = Oracle;

    fn as_ref(&self) -> OracleValueRef<'_> {
        OracleValueRef {
            value: self.value.as_deref(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, OracleTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}

impl<'r> ValueRef<'r> for OracleValueRef<'r> {
    type Database = Oracle;

    fn to_owned(&self) -> OracleValue {
        OracleValue {
            value: self.value.map(|v| v.to_vec()),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, OracleTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        self.value.is_none()
    }
}
