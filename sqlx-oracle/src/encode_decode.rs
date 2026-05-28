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

impl<'q> Encode<'q, Oracle> for f32 {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Float(*self as f64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for isize {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
        Ok(IsNull::No)
    }
}

impl<'q> Encode<'q, Oracle> for usize {
    fn encode_by_ref(&self, buf: &mut OracleArgumentBuffer) -> Result<IsNull, BoxDynError> {
        buf.push(OracleBindValue::Int(*self as i64));
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

impl<'r> Decode<'r, Oracle> for f32 {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        let v: f64 = s.parse::<f64>().map_err(|e| BoxDynError::from(format!("failed to parse f32: {e}")))?;
        Ok(v as f32)
    }
}

impl<'r> Decode<'r, Oracle> for isize {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse isize: {e}").into())
    }
}

impl<'r> Decode<'r, Oracle> for usize {
    fn decode(value: OracleValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = decode_text(value)?;
        s.parse().map_err(|e| format!("failed to parse usize: {e}").into())
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

impl Type<Oracle> for f32 {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::BinaryFloat
    }
}

impl Type<Oracle> for isize {
    fn type_info() -> OracleTypeInfo {
        OracleTypeInfo::Number
    }
}

impl Type<Oracle> for usize {
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
    if compact.len() % 2 != 0 {
        return Err("hex string has odd length".into());
    }
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

#[cfg(feature = "chrono")]
mod chrono_impls {
    //! chrono 时间类型 —— 连接时 NLS 已设为 ISO 8601 格式
    //!
    //! NaiveDateTime → TIMESTAMP
    //! NaiveDate     → DATE（可能包含时间部分，尝试两种格式）
    //! NaiveTime     → DATE（Oracle 总是返回完整时间戳，取时间部分）
    //! DateTime<Utc> → TIMESTAMP WITH TIME ZONE

    use super::decode_text;
    use crate::Oracle;
    use crate::OracleTypeInfo;
    use crate::arguments::{OracleArgumentBuffer, OracleBindValue};
    use crate::value::OracleValueRef;
    use sqlx_core::decode::Decode;
    use sqlx_core::encode::{Encode, IsNull};
    use sqlx_core::error::BoxDynError;
    use sqlx_core::types::Type;

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
            // 优先：有时区信息
            if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f %:z") {
                return Ok(dt.with_timezone(&chrono::Utc));
            }
            // 无时区：先解析为 NaiveDateTime，再假定为 UTC
            let naive = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S"))?;
            Ok(chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc))
        }
    }

    impl Type<Oracle> for chrono::DateTime<chrono::Utc> {
        fn type_info() -> OracleTypeInfo {
            OracleTypeInfo::TimestampTZ
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OracleArguments;
    use crate::OracleValueRef;
    use sqlx_core::arguments::Arguments;

    #[cfg(feature = "chrono")]
    use chrono::{Datelike, Timelike};

    // -----------------------------------------------------------------------
    // 辅助函数
    // -----------------------------------------------------------------------

    /// 编码一个值，返回产生的 `OracleBindValue`。
    fn encode_value<T: for<'q> Encode<'q, Oracle>>(value: T) -> OracleBindValue {
        let mut buf = OracleArgumentBuffer::default();
        let _ = Encode::encode(value, &mut buf).unwrap();
        buf.values.into_iter().next().unwrap()
    }

    /// 编码一个值的引用，返回产生的 `OracleBindValue`。
    fn encode_ref<T: for<'q> Encode<'q, Oracle>>(value: &T) -> OracleBindValue {
        let mut buf = OracleArgumentBuffer::default();
        let _ = Encode::encode_by_ref(value, &mut buf).unwrap();
        buf.values.into_iter().next().unwrap()
    }

    /// 将文本字节解码为指定类型。
    fn decode_text_as<T>(text: &str, type_info: OracleTypeInfo) -> T
    where
        T: for<'r> Decode<'r, Oracle>,
    {
        let value_ref = OracleValueRef {
            value: Some(text.as_bytes()),
            type_info,
        };
        Decode::decode(value_ref).unwrap()
    }

    /// 将二进制字节解码为指定类型（用于 Vec<u8> 的 RAW 分支）。
    fn decode_bytes_as<T>(bytes: &[u8], type_info: OracleTypeInfo) -> T
    where
        T: for<'r> Decode<'r, Oracle>,
    {
        let value_ref = OracleValueRef {
            value: Some(bytes),
            type_info,
        };
        Decode::decode(value_ref).unwrap()
    }

    /// 编码后再解码，验证 roundtrip 一致性。
    fn roundtrip<T>(value: T, type_info: OracleTypeInfo)
    where
        T: for<'q> Encode<'q, Oracle> + for<'r> Decode<'r, Oracle> + PartialEq + std::fmt::Debug,
    {
        let mut buf = OracleArgumentBuffer::default();
        let _ = Encode::encode_by_ref(&value, &mut buf).unwrap();
        let bind_value = buf.values.into_iter().next().unwrap();

        // 根据编码值重建文本并解码
        let text = match &bind_value {
            OracleBindValue::Null => panic!("unexpected Null in roundtrip"),
            OracleBindValue::String(s) => s.clone(),
            OracleBindValue::Int(i) => i.to_string(),
            OracleBindValue::Float(f) => f.to_string(),
            OracleBindValue::Bool(b) => (if *b { "1" } else { "0" }).to_string(),
        };

        let decoded: T = decode_text_as(&text, type_info);
        assert_eq!(decoded, value, "roundtrip failed for type");
    }

    // -----------------------------------------------------------------------
    // 字符串类型
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_string() {
        let v = encode_value("hello".to_string());
        assert!(matches!(&v, OracleBindValue::String(s) if s == "hello"));
    }

    #[test]
    fn test_encode_str_ref() {
        let v = encode_ref(&"hello");
        assert!(matches!(&v, OracleBindValue::String(s) if s == "hello"));
    }

    #[test]
    fn test_decode_string() {
        let decoded: String = decode_text_as("hello", OracleTypeInfo::Varchar2);
        assert_eq!(decoded, "hello");
    }

    #[test]
    fn test_decode_string_empty() {
        let decoded: String = decode_text_as("", OracleTypeInfo::Varchar2);
        assert_eq!(decoded, "");
    }

    #[test]
    fn test_string_roundtrip() {
        roundtrip("你好, Oracle!".to_string(), OracleTypeInfo::Varchar2);
        roundtrip(String::new(), OracleTypeInfo::Varchar2);
    }

    // -----------------------------------------------------------------------
    // 有符号整数
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_i64() {
        let v = encode_value(42i64);
        assert!(matches!(&v, OracleBindValue::Int(42)));
    }

    #[test]
    fn test_decode_i64() {
        let decoded: i64 = decode_text_as("42", OracleTypeInfo::Number);
        assert_eq!(decoded, 42);
    }

    #[test]
    fn test_i64_roundtrip() {
        roundtrip(0i64, OracleTypeInfo::Number);
        roundtrip(1i64, OracleTypeInfo::Number);
        roundtrip(-1i64, OracleTypeInfo::Number);
        roundtrip(i64::MAX, OracleTypeInfo::Number);
        roundtrip(i64::MIN, OracleTypeInfo::Number);
    }

    #[test]
    fn test_i32_roundtrip() {
        roundtrip(0i32, OracleTypeInfo::Number);
        roundtrip(42i32, OracleTypeInfo::Number);
        roundtrip(-42i32, OracleTypeInfo::Number);
        roundtrip(i32::MAX, OracleTypeInfo::Number);
        roundtrip(i32::MIN, OracleTypeInfo::Number);
    }

    #[test]
    fn test_i16_roundtrip() {
        roundtrip(0i16, OracleTypeInfo::Number);
        roundtrip(32767i16, OracleTypeInfo::Number);
        roundtrip(-32768i16, OracleTypeInfo::Number);
    }

    #[test]
    fn test_i8_roundtrip() {
        roundtrip(0i8, OracleTypeInfo::Number);
        roundtrip(127i8, OracleTypeInfo::Number);
        roundtrip(-128i8, OracleTypeInfo::Number);
    }

    #[test]
    fn test_isize_roundtrip() {
        roundtrip(0isize, OracleTypeInfo::Number);
        roundtrip(42isize, OracleTypeInfo::Number);
        roundtrip(-42isize, OracleTypeInfo::Number);
    }

    // -----------------------------------------------------------------------
    // 无符号整数
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_u64() {
        let v = encode_value(42u64);
        assert!(matches!(&v, OracleBindValue::Int(42)));
    }

    #[test]
    fn test_decode_u64() {
        let decoded: u64 = decode_text_as("42", OracleTypeInfo::Number);
        assert_eq!(decoded, 42);
    }

    #[test]
    fn test_u64_roundtrip() {
        roundtrip(0u64, OracleTypeInfo::Number);
        roundtrip(1u64, OracleTypeInfo::Number);
        roundtrip(i64::MAX as u64, OracleTypeInfo::Number);
    }

    #[test]
    fn test_u32_roundtrip() {
        roundtrip(0u32, OracleTypeInfo::Number);
        roundtrip(u32::MAX, OracleTypeInfo::Number);
    }

    #[test]
    fn test_u16_roundtrip() {
        roundtrip(0u16, OracleTypeInfo::Number);
        roundtrip(u16::MAX, OracleTypeInfo::Number);
    }

    #[test]
    fn test_u8_roundtrip() {
        roundtrip(0u8, OracleTypeInfo::Number);
        roundtrip(255u8, OracleTypeInfo::Number);
    }

    #[test]
    fn test_usize_roundtrip() {
        roundtrip(0usize, OracleTypeInfo::Number);
        roundtrip(42usize, OracleTypeInfo::Number);
    }

    // -----------------------------------------------------------------------
    // 浮点数
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_f64() {
        let v = encode_value(3.14f64);
        assert!(matches!(&v, OracleBindValue::Float(f) if (*f - 3.14).abs() < 1e-10));
    }

    #[test]
    fn test_decode_f64() {
        let decoded: f64 = decode_text_as("3.14", OracleTypeInfo::Number);
        assert!((decoded - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_f64_roundtrip() {
        roundtrip(0.0f64, OracleTypeInfo::Number);
        roundtrip(-0.0f64, OracleTypeInfo::Number);
        roundtrip(1.0f64, OracleTypeInfo::Number);
        roundtrip(-1.0f64, OracleTypeInfo::Number);
        roundtrip(3.141592653589793f64, OracleTypeInfo::Number);
        roundtrip(f64::MIN, OracleTypeInfo::Number);
        roundtrip(f64::MAX, OracleTypeInfo::Number);
    }

    #[test]
    fn test_f32_roundtrip() {
        roundtrip(0.0f32, OracleTypeInfo::BinaryFloat);
        roundtrip(1.0f32, OracleTypeInfo::BinaryFloat);
        roundtrip(-1.0f32, OracleTypeInfo::BinaryFloat);
        roundtrip(3.14f32, OracleTypeInfo::BinaryFloat);
        roundtrip(f32::MIN, OracleTypeInfo::BinaryFloat);
        roundtrip(f32::MAX, OracleTypeInfo::BinaryFloat);
    }

    #[test]
    fn test_decode_f64_inf_nan() {
        // Oracle 不直接支持 NaN/Infinity，但需要确保不会 panic
        let decoded: f64 = decode_text_as("NaN", OracleTypeInfo::Number);
        assert!(decoded.is_nan());

        let decoded: f64 = decode_text_as("Infinity", OracleTypeInfo::Number);
        assert!(decoded.is_infinite() && decoded.is_sign_positive());

        let decoded: f64 = decode_text_as("-Infinity", OracleTypeInfo::Number);
        assert!(decoded.is_infinite() && decoded.is_sign_negative());
    }

    // -----------------------------------------------------------------------
    // 布尔值
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_bool_true() {
        let v = encode_value(true);
        assert!(matches!(&v, OracleBindValue::Bool(true)));
    }

    #[test]
    fn test_encode_bool_false() {
        let v = encode_value(false);
        assert!(matches!(&v, OracleBindValue::Bool(false)));
    }

    #[test]
    fn test_decode_bool() {
        assert!(decode_text_as::<bool>("1", OracleTypeInfo::Number));
        assert!(decode_text_as::<bool>("true", OracleTypeInfo::Number));
        assert!(decode_text_as::<bool>("TRUE", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("0", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("false", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("FALSE", OracleTypeInfo::Number));
    }

    #[test]
    fn test_bool_roundtrip() {
        roundtrip(true, OracleTypeInfo::Number);
        roundtrip(false, OracleTypeInfo::Number);
    }

    // -----------------------------------------------------------------------
    // 二进制
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_vec_u8() {
        let v = encode_value(vec![0x00, 0xFF, 0xAB]);
        assert!(matches!(&v, OracleBindValue::String(s) if s == "00FFAB"));
    }

    #[test]
    fn test_decode_vec_u8_raw() {
        let decoded: Vec<u8> = decode_bytes_as(&[0x00, 0xFF, 0xAB], OracleTypeInfo::Raw);
        assert_eq!(decoded, vec![0x00, 0xFF, 0xAB]);
    }

    #[test]
    fn test_decode_vec_u8_long_raw() {
        let decoded: Vec<u8> = decode_bytes_as(&[0xDE, 0xAD], OracleTypeInfo::LongRaw);
        assert_eq!(decoded, vec![0xDE, 0xAD]);
    }

    #[test]
    fn test_decode_vec_u8_blob() {
        let decoded: Vec<u8> = decode_bytes_as(b"hello", OracleTypeInfo::Blob);
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn test_decode_vec_u8_hex_text() {
        // 对于非 RAW 类型，Vec<u8> 会 hex 解码文本
        let decoded: Vec<u8> = decode_text_as("00FFAB", OracleTypeInfo::Varchar2);
        assert_eq!(decoded, vec![0x00, 0xFF, 0xAB]);
    }

    #[test]
    fn test_decode_vec_u8_hex_text_lowercase() {
        let decoded: Vec<u8> = decode_text_as("00ffab", OracleTypeInfo::Varchar2);
        assert_eq!(decoded, vec![0x00, 0xFF, 0xAB]);
    }

    #[test]
    fn test_vec_u8_roundtrip_hex() {
        // Vec<u8> 通过 hex 文本编解码（非 RAW 分支）
        roundtrip(vec![], OracleTypeInfo::Varchar2);
        roundtrip(vec![0x00, 0xFF], OracleTypeInfo::Varchar2);
    }

    #[test]
    fn test_vec_u8_empty() {
        let v = encode_value(Vec::<u8>::new());
        assert!(matches!(&v, OracleBindValue::String(s) if s.is_empty()));
    }

    // -----------------------------------------------------------------------
    // 空值
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_null_returns_error() {
        let value_ref = OracleValueRef {
            value: None,
            type_info: OracleTypeInfo::Varchar2,
        };
        let result: Result<String, BoxDynError> = Decode::decode(value_ref);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Option 编码
    // -----------------------------------------------------------------------

    #[test]
    fn test_encode_option_some() {
        let mut buf = OracleArgumentBuffer::default();
        let _ = Encode::encode(Some(42i64), &mut buf).unwrap();
        assert!(matches!(buf.values[0], OracleBindValue::Int(42)));
    }

    #[test]
    fn test_encode_option_none() {
        let mut buf = OracleArgumentBuffer::default();
        let result = Encode::encode(None::<i64>, &mut buf);
        assert!(matches!(result, Ok(IsNull::Yes)));
    }

    // -----------------------------------------------------------------------
    // 编码结果验证
    // -----------------------------------------------------------------------

    #[test]
    fn test_type_info_mappings() {
        assert_eq!(<String as Type<Oracle>>::type_info(), OracleTypeInfo::Varchar2);
        assert_eq!(<&str as Type<Oracle>>::type_info(), OracleTypeInfo::Varchar2);
        assert_eq!(<i64 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<i32 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<i16 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<i8 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<u64 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<u32 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<u16 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<u8 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<f64 as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<f32 as Type<Oracle>>::type_info(), OracleTypeInfo::BinaryFloat);
        assert_eq!(<isize as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<usize as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<bool as Type<Oracle>>::type_info(), OracleTypeInfo::Number);
        assert_eq!(<Vec<u8> as Type<Oracle>>::type_info(), OracleTypeInfo::Raw);
        #[cfg(feature = "chrono")]
        assert_eq!(<chrono::NaiveDateTime as Type<Oracle>>::type_info(), OracleTypeInfo::Timestamp);
        #[cfg(feature = "chrono")]
        assert_eq!(<chrono::NaiveDate as Type<Oracle>>::type_info(), OracleTypeInfo::Date);
        #[cfg(feature = "chrono")]
        assert_eq!(<chrono::NaiveTime as Type<Oracle>>::type_info(), OracleTypeInfo::Date);
        #[cfg(feature = "chrono")]
        assert_eq!(<chrono::DateTime<chrono::Utc> as Type<Oracle>>::type_info(), OracleTypeInfo::TimestampTZ);
    }

    // -----------------------------------------------------------------------
    // 批量边界值测试
    // -----------------------------------------------------------------------

    #[test]
    fn test_integer_boundaries() {
        // 验证编码不 panic
        let cases: Vec<i64> = vec![
            i64::MIN,
            i64::MIN + 1,
            -1,
            0,
            1,
            i64::MAX - 1,
            i64::MAX,
        ];
        for v in cases {
            let _ = encode_value(v);
        }
    }

    #[test]
    fn test_unsigned_boundaries() {
        let cases: Vec<u64> = vec![0, 1, u64::MAX - 1, u64::MAX];
        for v in cases {
            let _ = encode_value(v);
        }
    }

    #[test]
    fn test_float_special_values() {
        // 验证 NaN/Infinity 编码不会 panic
        let _ = encode_value(f64::NAN);
        let _ = encode_value(f64::INFINITY);
        let _ = encode_value(f64::NEG_INFINITY);
    }

    #[test]
    fn test_i64_decode_negative() {
        let decoded: i64 = decode_text_as("-9223372036854775808", OracleTypeInfo::Number);
        assert_eq!(decoded, i64::MIN);
    }

    // -----------------------------------------------------------------------
    // hex 编解码
    // -----------------------------------------------------------------------

    #[test]
    fn test_hex_encode_decode() {
        assert_eq!(hex_encode(&[0x00, 0xFF, 0xAB]), "00FFAB");
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex_decode("00FFAB").unwrap(), vec![0x00, 0xFF, 0xAB]);
        assert_eq!(hex_decode("00ffab").unwrap(), vec![0x00, 0xFF, 0xAB]);
        assert_eq!(hex_decode("").unwrap(), vec![]);
        assert!(hex_decode("0").is_err(), "odd-length input should error");
        assert!(hex_decode("XX").is_err());
    }

    #[test]
    fn test_hex_decode_with_whitespace() {
        assert_eq!(hex_decode("00 FF AB").unwrap(), vec![0x00, 0xFF, 0xAB]);
        assert_eq!(hex_decode(" 00ff ").unwrap(), vec![0x00, 0xFF]);
    }

    #[test]
    fn test_hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_encode_full_range() {
        let bytes: Vec<u8> = (0..=255).collect();
        let encoded = hex_encode(&bytes);
        assert_eq!(encoded.len(), 512);
        assert!(encoded.starts_with("000102"));
        assert!(encoded.ends_with("FDFEFF"));
    }

    #[test]
    fn test_string_special_chars() {
        let s = "\n\t\r\\'\0";
        let v = encode_value(s.to_string());
        assert!(matches!(&v, OracleBindValue::String(x) if x == s));
    }

    #[test]
    fn test_float_negative() {
        roundtrip(-3.14f64, OracleTypeInfo::Number);
        roundtrip(-0.001f64, OracleTypeInfo::Number);
        roundtrip(-1.0e-10f64, OracleTypeInfo::Number);
    }

    #[test]
    fn test_float_small() {
        roundtrip(1e-100f64, OracleTypeInfo::Number);
        roundtrip(1e100f64, OracleTypeInfo::Number);
    }

    #[test]
    fn test_f32_special_encode() {
        let _ = encode_value(f32::INFINITY);
        let _ = encode_value(f32::NEG_INFINITY);
        let _ = encode_value(f32::NAN);
    }

    #[test]
    fn test_bool_decode_variants() {
        assert!(decode_text_as::<bool>("1", OracleTypeInfo::Number));
        assert!(decode_text_as::<bool>("true", OracleTypeInfo::Number));
        assert!(decode_text_as::<bool>("TRUE", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("0", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("false", OracleTypeInfo::Number));
        assert!(!decode_text_as::<bool>("FALSE", OracleTypeInfo::Number));
    }

    #[test]
    fn test_bool_decode_invalid_error() {
        let value_ref = OracleValueRef {
            value: Some(b"2"),
            type_info: OracleTypeInfo::Number,
        };
        let result: Result<bool, BoxDynError> = Decode::decode(value_ref);
        assert!(result.is_err());
    }

    #[test]
    fn test_i64_u64_boundary() {
        let v = i64::MAX as u64;
        roundtrip(v, OracleTypeInfo::Number);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_naive_datetime_subsecond_edge_cases() {
        let dt1 = chrono::NaiveDateTime::parse_from_str("2024-01-01 00:00:00.000000", "%Y-%m-%d %H:%M:%S%.f").unwrap();
        let dt2 = chrono::NaiveDateTime::parse_from_str("2024-01-01 00:00:00.999999", "%Y-%m-%d %H:%M:%S%.f").unwrap();
        roundtrip(dt1, OracleTypeInfo::Timestamp);
        roundtrip(dt2, OracleTypeInfo::Timestamp);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_naive_date_leap_year() {
        roundtrip(chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(), OracleTypeInfo::Date);
        roundtrip(chrono::NaiveDate::from_ymd_opt(2000, 2, 29).unwrap(), OracleTypeInfo::Date);
        roundtrip(chrono::NaiveDate::from_ymd_opt(1900, 2, 28).unwrap(), OracleTypeInfo::Date);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_datetime_utc_positive_offset() {
        let s = "2024-01-15 18:30:45 +08:00";
        let decoded: chrono::DateTime<chrono::Utc> = decode_text_as(s, OracleTypeInfo::TimestampTZ);
        assert_eq!(decoded.hour(), 10);
        assert_eq!(decoded.minute(), 30);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_datetime_utc_negative_offset() {
        let s = "2024-01-15 06:30:45 -05:00";
        let decoded: chrono::DateTime<chrono::Utc> = decode_text_as(s, OracleTypeInfo::TimestampTZ);
        assert_eq!(decoded.hour(), 11);
        assert_eq!(decoded.minute(), 30);
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_datetime_utc_with_fractional_tz() {
        let s = "2024-01-15 10:15:45 +05:30";
        let decoded: chrono::DateTime<chrono::Utc> = decode_text_as(s, OracleTypeInfo::TimestampTZ);
        assert_eq!(decoded.hour(), 4);
        assert_eq!(decoded.minute(), 45);
    }

    #[test]
    fn test_u64_overflow_encoding() {
        let mut buf = OracleArgumentBuffer::default();
        // u64::MAX as i64 wraps to -1
        let val = u64::MAX;
        let _ = Encode::encode_by_ref(&val, &mut buf).unwrap();
        assert!(matches!(buf.values[0], OracleBindValue::Int(-1)));
    }

    #[test]
    fn test_encode_multiple_values() {
        let mut buf = OracleArgumentBuffer::default();
        let _ = Encode::encode_by_ref(&1i64, &mut buf).unwrap();
        let _ = Encode::encode_by_ref(&"hello", &mut buf).unwrap();
        let _ = Encode::encode_by_ref(&3.14f64, &mut buf).unwrap();
        let _ = Encode::encode_by_ref(&true, &mut buf).unwrap();
        assert_eq!(buf.values.len(), 4);
        assert!(matches!(buf.values[0], OracleBindValue::Int(1)));
        assert!(matches!(&buf.values[1], OracleBindValue::String(s) if s == "hello"));
        assert!(matches!(buf.values[2], OracleBindValue::Float(f) if (f - 3.14).abs() < 1e-10));
        assert!(matches!(buf.values[3], OracleBindValue::Bool(true)));
    }

    #[test]
    fn test_format_placeholder() {
        let args = OracleArguments::default();
        let mut output = String::new();
        args.format_placeholder(&mut output).unwrap();
        assert_eq!(output, "$0");
    }

}
