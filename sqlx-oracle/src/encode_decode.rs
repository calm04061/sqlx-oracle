use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

use crate::arguments::OracleBindValue;
use crate::{Oracle, OracleArgumentBuffer, OracleTypeInfo, OracleValueRef};

impl<'q> Encode<'q, Oracle> for &str {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String((*self).to_owned()));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for String {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.clone()));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for i64 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for i32 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for f64 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Float(*self));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for bool {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Bool(*self));
        Ok(IsNull::No)
    }
}



fn decode_text(value: OracleValueRef<'_>) -> Result<String, BoxDynError> {
    let bytes = value.value.ok_or("unexpected null")?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

impl<'r> Decode<'r, Oracle> for String {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        decode_text(value)
    }
}

impl<'r> Decode<'r, Oracle> for i64 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse i64: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for i32 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse i32: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for f64 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse f64: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for bool {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        match s.as_str() {
            "1" | "true" | "TRUE" => Ok(true),
            "0" | "false" | "FALSE" => Ok(false),
            _ => Err(format!("failed to parse bool: {s}").into()),
        }
    }
}

impl Type<Oracle> for String {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Varchar2
    }
}

impl Type<Oracle> for &str {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Varchar2
    }
}

impl Type<Oracle> for i64 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for i32 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for f64 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for bool {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Boolean
    }
}

sqlx_core::impl_encode_for_option!(Oracle);
