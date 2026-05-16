use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

use crate::arguments::OracleBindValue;
use crate::{Oracle, OracleArgumentBuffer, OracleTypeInfo, OracleValueRef};

// ===========================================================================
// Encode 实现：Rust 值 → Oracle 绑定值
// ===========================================================================

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

impl<'q> Encode<'q, Oracle> for i16 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for i8 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for u64 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for u32 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for u16 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for u8 {
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

// ===========================================================================
// Decode 实现：Oracle 文本值 → Rust 值
// ===========================================================================

/// 从列值中解码文本内容。
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

impl<'r> Decode<'r, Oracle> for i16 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse i16: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for i8 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse i8: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for u64 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse u64: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for u32 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse u32: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for u16 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse u16: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for u8 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse u8: {e}").into())
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

// ===========================================================================
// Type 映射：Rust 类型 → Oracle 数据类型
// ===========================================================================

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

impl Type<Oracle> for i16 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for i8 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for u64 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for u32 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for u16 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for u8 {
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
        // Oracle SQL 没有原生 BOOLEAN 类型，实际以 i32 (0/1) 编解码
        OracleTypeInfo::Number
    }
}

sqlx_core::impl_encode_for_option!(Oracle);

// ===========================================================================
// Vec<u8> —— 以十六进制字符串编码输出，RAW 二进制字节直接读取
// ===========================================================================

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
        match value.type_info {
            OracleTypeInfo::Raw | OracleTypeInfo::LongRaw | OracleTypeInfo::Blob => {
                value.value.ok_or_else(|| "unexpected null".into()).map(|v| v.to_vec())
            }
            _ => {
                let s = decode_text(value)?;
                hex_decode(&s)
            }
        }
    }
}

impl Type<Oracle> for Vec<u8> {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Raw
    }
}

// ===========================================================================
// chrono 时间类型 —— 连接时 NLS 已设为 ISO 8601 格式
// ===========================================================================
//
// NaiveDateTime → TIMESTAMP
// NaiveDate     → DATE（可能包含时间部分，尝试两种格式）
// NaiveTime     → DATE（Oracle 总是返回完整时间戳，取时间部分）
// DateTime<Utc> → TIMESTAMP WITH TIME ZONE

fn fmt_subsec_micros(nanos: u32) -> String {
    if nanos == 0 {
        String::new()
    } else {
        format!(".{:06}", nanos / 1000)
    }
}

impl<'q> Encode<'q, Oracle> for chrono::NaiveDateTime {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let nanos = self.and_utc().timestamp_subsec_nanos();
        let s = format!("{}{}", self.format("%Y-%m-%d %H:%M:%S"), fmt_subsec_micros(nanos));
        buf.push(OracleBindValue::String(s));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let nanos = self.and_utc().timestamp_subsec_nanos();
        let s = format!("{}{}", self.format("%Y-%m-%d %H:%M:%S"), fmt_subsec_micros(nanos));
        buf.push(OracleBindValue::String(s));
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

impl<'q> Encode<'q, Oracle> for chrono::NaiveTime {
    fn encode(self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("2000-01-01 %H:%M:%S").to_string()));
        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::String(self.format("2000-01-01 %H:%M:%S").to_string()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Oracle> for chrono::NaiveTime {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
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
