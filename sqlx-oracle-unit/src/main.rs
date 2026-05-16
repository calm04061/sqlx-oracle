//! sqlx-oracle 集成测试。
//!
//! 通过连接真实 Oracle 数据库运行，覆盖各种数据类型和 SQL 操作。
//! 默认连接字符串从 `DATABASE_URL` 环境变量读取。
//! 要求已安装 Oracle Instant Client。

use sqlx::Connection;
use sqlx_oracle::OracleConnection;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "oracle://scott:tiger@localhost:1521/FREEPDB1".to_string());
    println!("connecting to {url} ...");

    let mut conn = OracleConnection::connect(&url).await.unwrap();
    println!("connected\n");

    println!("=== test_select_scalar ===");
    test_select_scalar(&mut conn).await;

    println!("\n=== test_select_params ===");
    test_select_params(&mut conn).await;

    println!("\n=== test_i64 ===");
    test_i64(&mut conn).await;

    println!("\n=== test_f64 ===");
    test_f64(&mut conn).await;

    println!("\n=== test_bool ===");
    test_bool(&mut conn).await;

    println!("\n=== test_string ===");
    test_string(&mut conn).await;

    println!("\n=== test_insert_select ===");
    test_insert_select(&mut conn).await;

    println!("\n=== test_update ===");
    test_update(&mut conn).await;

    println!("\n=== test_transaction_rollback ===");
    test_transaction_rollback(&mut conn).await;

    println!("\n=== test_bytes ===");
    test_bytes(&mut conn).await;

    println!("\n=== test_chrono ===");
    test_chrono(&mut conn).await;

    println!("\n=== test_null_binding ===");
    test_null_binding(&mut conn).await;

    println!("\nall tests passed");
}

use sqlx::Row;

/// 测试基础标量查询和列名索引。
async fn test_select_scalar(conn: &mut OracleConnection) {
    let row = sqlx::query("SELECT 1 AS n, 'hello' AS s FROM DUAL")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let n: i64 = row.get("N");
    let s: String = row.get("S");
    assert_eq!(n, 1);
    assert_eq!(s, "hello");
    println!("  n={n} s={s}");
}

/// 测试带参数（占位符）的查询。
async fn test_select_params(conn: &mut OracleConnection) {
    let row = sqlx::query("SELECT ? + ? AS sum, ? AS msg FROM DUAL")
        .bind(3i64)
        .bind(4i64)
        .bind("hello")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let sum: i64 = row.get("SUM");
    let msg: String = row.get("MSG");
    assert_eq!(sum, 7);
    assert_eq!(msg, "hello");
    println!("  sum={sum} msg={msg}");
}

/// 测试 i64 类型的编码和解码（正数、负数、最大值）。
async fn test_i64(conn: &mut OracleConnection) {
    for (input, label) in [(42i64, "positive"), (-1, "negative"), (i64::MAX, "MAX")] {
        let row = sqlx::query("SELECT ? AS v FROM DUAL")
            .bind(input)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        let v: i64 = row.get("V");
        assert_eq!(v, input);
        println!("  {label}: {v}");
    }
}

/// 测试 f64 类型的编码和解码。
async fn test_f64(conn: &mut OracleConnection) {
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(2.5f64)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: f64 = row.get("V");
    assert!((v - 2.5).abs() < 1e-6);
    println!("  f64: {v}");
}

/// 测试 bool 类型的编码和解码。
async fn test_bool(conn: &mut OracleConnection) {
    for input in [true, false] {
        let row = sqlx::query("SELECT ? AS v FROM DUAL")
            .bind(input)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        let v: bool = row.get("V");
        assert_eq!(v, input);
        println!("  {input}: {v}");
    }
}

/// 测试字符串类型的编码和解码。
async fn test_string(conn: &mut OracleConnection) {
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind("the quick brown fox")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: String = row.get("V");
    assert_eq!(v, "the quick brown fox");
    println!("  string: {v}");

    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind("")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: Result<String, _> = row.try_get("V");
    println!("  empty string: result={v:?}");
}

/// 测试 INSERT 后 SELECT 的完整 DML 流程。
async fn test_insert_select(conn: &mut OracleConnection) {
    sqlx::query("CREATE TABLE sqlx_test_insel (id NUMBER, label VARCHAR2(100))")
        .execute(&mut *conn)
        .await
        .unwrap();

    sqlx::query("INSERT INTO sqlx_test_insel (id, label) VALUES (?, ?)")
        .bind(1i64)
        .bind("alpha")
        .execute(&mut *conn)
        .await
        .unwrap();
    println!("  inserted 1 row");

    let row = sqlx::query("SELECT id, label FROM sqlx_test_insel WHERE id = ?")
        .bind(1i64)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let id: i64 = row.get("ID");
    let label: String = row.get("LABEL");
    assert_eq!(id, 1);
    assert_eq!(label, "alpha");
    println!("  read back: id={id} label={label}");

    sqlx::query("DROP TABLE sqlx_test_insel")
        .execute(&mut *conn)
        .await
        .unwrap();
    println!("  cleaned up");
}

/// 测试 UPDATE 语句及 rows_affected()。
async fn test_update(conn: &mut OracleConnection) {
    sqlx::query("CREATE TABLE sqlx_test_upd (id NUMBER, val NUMBER)")
        .execute(&mut *conn)
        .await
        .unwrap();

    sqlx::query("INSERT INTO sqlx_test_upd (id, val) VALUES (?, ?)")
        .bind(1i64)
        .bind(10i64)
        .execute(&mut *conn)
        .await
        .unwrap();

    let affected = sqlx::query("UPDATE sqlx_test_upd SET val = ? WHERE id = ?")
        .bind(99i64)
        .bind(1i64)
        .execute(&mut *conn)
        .await
        .unwrap()
        .rows_affected();
    assert_eq!(affected, 1);
    println!("  updated {affected} row(s)");

    sqlx::query("DROP TABLE sqlx_test_upd")
        .execute(&mut *conn)
        .await
        .unwrap();
}

/// 测试事务回滚：插入后回滚，验证行不存在。
async fn test_transaction_rollback(conn: &mut OracleConnection) {
    sqlx::query("CREATE TABLE sqlx_test_txn (id NUMBER)")
        .execute(&mut *conn)
        .await
        .unwrap();

    let mut tx = conn.begin().await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_txn (id) VALUES (?)")
        .bind(42i64)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    println!("  rolled back");

    let row = sqlx::query("SELECT COUNT(*) AS cnt FROM sqlx_test_txn WHERE id = ?")
        .bind(42i64)
        .fetch_optional(&mut *conn)
        .await
        .unwrap();
    let cnt: i64 = row.as_ref().map(|r| r.get("CNT")).unwrap_or(0);
    assert_eq!(cnt, 0, "rollback did not remove the row");
    println!("  rollback verified (cnt={cnt})");

    sqlx::query("DROP TABLE sqlx_test_txn")
        .execute(&mut *conn)
        .await
        .unwrap();
}

/// 测试二进制数据（Vec<u8>）的十六进制编解码。
async fn test_bytes(conn: &mut OracleConnection) {
    let payload = vec![0xde, 0xad, 0xbe, 0xefu8];
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(payload.clone())
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: Vec<u8> = row.get("V");
    assert_eq!(v, payload);
    println!("  bytes: {v:x?}");
}

/// 测试 chrono 时间类型的编解码。
async fn test_chrono(conn: &mut OracleConnection) {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

    let dt = NaiveDateTime::parse_from_str("2026-05-16 12:34:56.123456", "%Y-%m-%d %H:%M:%S%.f").unwrap();
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(dt)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: NaiveDateTime = row.get("V");
    assert_eq!(v, dt);
    println!("  NaiveDateTime: {v}");

    let d = NaiveDate::from_ymd_opt(2026, 5, 16).unwrap();
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(d)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: NaiveDate = row.get("V");
    assert_eq!(v, d);
    println!("  NaiveDate: {v}");

    let t = NaiveTime::from_hms_micro_opt(12, 34, 56, 123456).unwrap();
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(t)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: NaiveTime = row.get("V");
    assert_eq!(v, t);
    println!("  NaiveTime: {v}");

    let utc_dt = Utc.with_ymd_and_hms(2026, 5, 16, 12, 34, 56).unwrap();
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(utc_dt)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: chrono::DateTime<Utc> = row.get("V");
    assert_eq!(v, utc_dt);
    println!("  DateTime<Utc>: {v}");
}

/// 测试 NULL 绑定。
async fn test_null_binding(conn: &mut OracleConnection) {
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(None::<String>)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    let v: Option<String> = row.get("V");
    assert!(v.is_none());
    println!("  NULL: {v:?}");
}
