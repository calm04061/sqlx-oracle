use std::borrow::Cow;
use std::fmt::Debug;

use crate::OracleTypeInfo;
use crate::Oracle;
use sqlx_core::value::{Value, ValueRef};

/// 列值（拥有所有权）。
///
/// 以 `Option<Vec<u8>>` 形式存储原始字节数据，目前所有类型
/// 均通过文本格式编解码。
#[derive(Debug, Clone)]
pub struct OracleValue {
    pub value: Option<Vec<u8>>,
    pub type_info: OracleTypeInfo,
}

/// 列值引用。
///
/// 借自 `OracleRow` 中的值，用于 `Decode` 实现中零拷贝读取。
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::value::{Value, ValueRef};

    fn make_value(data: &[u8], type_info: OracleTypeInfo) -> OracleValue {
        OracleValue {
            value: Some(data.to_vec()),
            type_info,
        }
    }

    fn make_value_ref(data: &[u8], type_info: OracleTypeInfo) -> OracleValueRef<'_> {
        OracleValueRef {
            value: Some(data),
            type_info,
        }
    }

    fn make_null(type_info: OracleTypeInfo) -> OracleValue {
        OracleValue { value: None, type_info }
    }

    #[test]
    fn test_value_with_data() {
        let v = make_value(b"hello", OracleTypeInfo::Varchar2);
        assert!(!v.is_null());
        assert_eq!(v.type_info(), std::borrow::Cow::Borrowed(&OracleTypeInfo::Varchar2));
    }

    #[test]
    fn test_value_null() {
        let v = make_null(OracleTypeInfo::Null);
        assert!(v.is_null());
    }

    #[test]
    fn test_value_as_ref() {
        let v = make_value(b"data", OracleTypeInfo::Varchar2);
        let r = v.as_ref();
        assert_eq!(r.value, Some(b"data" as &[u8]));
        assert_eq!(r.type_info, OracleTypeInfo::Varchar2);
    }

    #[test]
    fn test_value_ref_to_owned() {
        let r = make_value_ref(b"test", OracleTypeInfo::Raw);
        let owned = ValueRef::to_owned(&r);
        assert_eq!(owned.value, Some(b"test".to_vec()));
        assert_eq!(owned.type_info, OracleTypeInfo::Raw);
    }

    #[test]
    fn test_value_ref_null() {
        let r = OracleValueRef { value: None, type_info: OracleTypeInfo::Null };
        assert!(r.is_null());
        let owned = ValueRef::to_owned(&r);
        assert!(owned.is_null());
    }

    #[test]
    fn test_value_ref_debug() {
        let r = make_value_ref(b"abc", OracleTypeInfo::Varchar2);
        let debug = format!("{r:?}");
        assert!(debug.contains("97") || debug.contains("value"));
        assert!(debug.contains("Varchar2"));
    }

    #[test]
    fn test_value_clone() {
        let a = make_value(b"clone", OracleTypeInfo::Number);
        let b = a.clone();
        assert_eq!(a.value, b.value);
        assert_eq!(a.type_info, b.type_info);
    }

    #[test]
    fn test_value_empty_bytes_not_null() {
        let v = make_value(b"", OracleTypeInfo::Varchar2);
        assert!(!v.is_null(), "empty bytes should not be null");
    }

    #[test]
    fn test_value_ref_empty_bytes() {
        let r = make_value_ref(b"", OracleTypeInfo::Varchar2);
        assert!(!r.is_null());
        let owned = ValueRef::to_owned(&r);
        assert_eq!(owned.value, Some(vec![]));
    }

    #[test]
    fn test_value_type_info_borrowed() {
        let v = make_value(b"x", OracleTypeInfo::Timestamp);
        match v.type_info() {
            std::borrow::Cow::Borrowed(_) => {} // expected
            _ => panic!("expected Borrowed Cow"),
        }
    }

    #[test]
    fn test_value_ref_type_info_borrowed() {
        let r = make_value_ref(b"x", OracleTypeInfo::Number);
        match r.type_info() {
            std::borrow::Cow::Borrowed(_) => {} // expected
            _ => panic!("expected Borrowed Cow"),
        }
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
