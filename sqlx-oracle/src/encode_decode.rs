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

// ---------------------------------------------------------------------------
// Vec<u8> — encoded as hex string (for RAW / BLOB columns)
// ---------------------------------------------------------------------------

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        write!(s, "{b:02X}").unwrap();
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, BoxDynError> {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut out = Vec::with_capacity(compact.len() / 2);
    let mut i = 0;
    let bytes = compact.as_bytes();
    while i + 1 < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_val(b: u8) -> Result<u8, BoxDynError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(format!("invalid hex char: {b}").into()),
    }
}

impl<'q> Encode<'q, Oracle> for Vec<u8> {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(hex_encode(&self)));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(hex_encode(self)));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for Vec<u8> {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        hex_decode(&s)
    }
}

impl Type<Oracle> for Vec<u8> {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Raw
    }
}

// ---------------------------------------------------------------------------
// chrono types — NLS set to ISO 8601 on connect
// ---------------------------------------------------------------------------

impl<'q> Encode<'q, Oracle> for chrono::NaiveDateTime {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d %H:%M:%S%.f").to_string()));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d %H:%M:%S%.f").to_string()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for chrono::NaiveDateTime {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
            .map_err(|e| format!("failed to parse NaiveDateTime: {e}").into())
    }
}

impl Type<Oracle> for chrono::NaiveDateTime {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Timestamp
    }
}

// ---- NaiveDate -----------------------------------------------------------

impl<'q> Encode<'q, Oracle> for chrono::NaiveDate {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d").to_string()));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d").to_string()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for chrono::NaiveDate {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        // Oracle DATE includes time with NLS set; try date-only first, then full datetime
        chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.date())
            })
            .map_err(|e| format!("failed to parse NaiveDate: {e}").into())
    }
}

impl Type<Oracle> for chrono::NaiveDate {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Date
    }
}

// ---- NaiveTime -----------------------------------------------------------

impl<'q> Encode<'q, Oracle> for chrono::NaiveTime {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%H:%M:%S%.f").to_string()));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%H:%M:%S%.f").to_string()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for chrono::NaiveTime {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        // Oracle gives "YYYY-MM-DD HH:MM:SS.FF" for DATE columns — try time-only first
        chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f")
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
                    .map(|dt| dt.time())
            })
            .map_err(|e| format!("failed to parse NaiveTime: {e}").into())
    }
}

impl Type<Oracle> for chrono::NaiveTime {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Date
    }
}

// ---- DateTime<Utc> -------------------------------------------------------

impl<'q> Encode<'q, Oracle> for chrono::DateTime<chrono::Utc> {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d %H:%M:%S%.f %:z").to_string()));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("%Y-%m-%d %H:%M:%S%.f %:z").to_string()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for chrono::DateTime<chrono::Utc> {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        chrono::DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f %:z")
            .or_else(|_| chrono::DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f"))
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| format!("failed to parse DateTime<Utc>: {e}").into())
    }
}

impl Type<Oracle> for chrono::DateTime<chrono::Utc> {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::TimestampTZ
    }
}
