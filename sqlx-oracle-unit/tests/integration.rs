//! sqlx-oracle 集成测试。
//!
//! 使用 `DATABASE_URL` 环境变量连接 Oracle，运行方式：
//! ```sh
//! cargo test -p sqlx-oracle-unit -- --test-threads=1 --nocapture
//! ```

use sqlx::{Connection, Row};

fn db_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "oracle://scott:tiger@localhost:1521/FREEPDB1".to_string())
}

async fn new_conn() -> sqlx_oracle::OracleConnection {
    sqlx_oracle::OracleConnection::connect(&db_url())
        .await
        .expect("connect to Oracle")
}

#[tokio::test]
async fn select_scalar() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT 1 AS n, 'hello' AS s FROM DUAL")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let n: i64 = row.get("N");
    let s: String = row.get("S");
    assert_eq!(n, 1);
    assert_eq!(s, "hello");
}

#[tokio::test]
async fn select_params() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT ? + ? AS sum, ? AS msg FROM DUAL")
        .bind(3i64)
        .bind(4i64)
        .bind("hello")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let sum: i64 = row.get("SUM");
    let msg: String = row.get("MSG");
    assert_eq!(sum, 7);
    assert_eq!(msg, "hello");
}

#[tokio::test]
async fn i64_roundtrip() {
    let mut conn = new_conn().await;
    for (input, _label) in [(42i64, "positive"), (-1, "negative"), (i64::MAX, "MAX")] {
        let row = sqlx::query("SELECT CAST(? AS NUMBER) AS v FROM DUAL")
            .bind(input)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        let v: i64 = row.get("V");
        assert_eq!(v, input);
    }
}

#[tokio::test]
async fn i8_roundtrip() {
    let mut conn = new_conn().await;
    let params: [(i8, &str); 3] = [(42, "positive"), (-1, "negative"), (i8::MAX, "MAX")];
    for (input, _label) in params {
        let row = sqlx::query("SELECT CAST(? AS NUMBER) AS v FROM DUAL")
            .bind(input)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        let v: i8 = row.get("V");
        assert_eq!(v, input);
    }
}

#[tokio::test]
async fn f64_roundtrip() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT CAST(? AS BINARY_DOUBLE) AS v FROM DUAL")
        .bind(2.5f64)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: f64 = row.get("V");
    assert!((v - 2.5).abs() < 1e-6);
}

#[tokio::test]
async fn bool_roundtrip() {
    let mut conn = new_conn().await;
    for input in [true, false] {
        let row = sqlx::query("SELECT CAST(? AS NUMBER) AS v FROM DUAL")
            .bind(input)
            .fetch_one(&mut conn)
            .await
            .unwrap();
        let v: bool = row.get("V");
        assert_eq!(v, input);
    }
}

#[tokio::test]
async fn string_roundtrip() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT CAST(? AS VARCHAR2(4000)) AS v FROM DUAL")
        .bind("the quick brown fox")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: String = row.get("V");
    assert_eq!(v, "the quick brown fox");

    let row = sqlx::query("SELECT CAST(? AS VARCHAR2(4000)) AS v FROM DUAL")
        .bind("")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: Option<String> = row.get("V");
    assert!(v.is_none(), "Oracle treats empty string as NULL");
}

#[tokio::test]
async fn insert_select() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_insel")
        .execute(&mut conn)
        .await;
    sqlx::query("CREATE TABLE sqlx_test_insel (id NUMBER, label VARCHAR2(100)) NOPARALLEL")
        .execute(&mut conn)
        .await
        .unwrap();

    sqlx::query("INSERT INTO sqlx_test_insel (id, label) VALUES (?, ?)")
        .bind(1i64)
        .bind("alpha")
        .execute(&mut conn)
        .await
        .unwrap();

    let row = sqlx::query("SELECT id, label FROM sqlx_test_insel WHERE id = ?")
        .bind(1i64)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let id: i64 = row.get("ID");
    let label: String = row.get("LABEL");
    assert_eq!(id, 1);
    assert_eq!(label, "alpha");

    sqlx::query("DROP TABLE sqlx_test_insel")
        .execute(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn update() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_upd")
        .execute(&mut conn)
        .await;
    sqlx::query("CREATE TABLE sqlx_test_upd (id NUMBER, val NUMBER) NOPARALLEL")
        .execute(&mut conn)
        .await
        .unwrap();

    sqlx::query("INSERT INTO sqlx_test_upd (id, val) VALUES (?, ?)")
        .bind(1i64)
        .bind(10i64)
        .execute(&mut conn)
        .await
        .unwrap();

    sqlx::query("COMMIT").execute(&mut conn).await.unwrap();

    let affected = sqlx::query("UPDATE sqlx_test_upd SET val = ? WHERE id = ?")
        .bind(99i64)
        .bind(1i64)
        .execute(&mut conn)
        .await
        .unwrap()
        .rows_affected();
    assert_eq!(affected, 1);

    sqlx::query("DROP TABLE sqlx_test_upd")
        .execute(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn transaction_rollback() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_txn")
        .execute(&mut conn)
        .await;
    sqlx::query("CREATE TABLE sqlx_test_txn (id NUMBER) NOPARALLEL")
        .execute(&mut conn)
        .await
        .unwrap();

    let mut tx = conn.begin().await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_txn (id) VALUES (?)")
        .bind(42i64)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();

    let row = sqlx::query("SELECT COUNT(*) AS cnt FROM sqlx_test_txn WHERE id = ?")
        .bind(42i64)
        .fetch_optional(&mut conn)
        .await
        .unwrap();
    let cnt: i64 = row.as_ref().map(|r| r.get("CNT")).unwrap_or(0);
    assert_eq!(cnt, 0, "rollback did not remove the row");

    sqlx::query("DROP TABLE sqlx_test_txn")
        .execute(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn bytes_roundtrip() {
    let mut conn = new_conn().await;
    let payload = vec![0xde, 0xad, 0xbe, 0xefu8];
    let row = sqlx::query("SELECT CAST(? AS RAW(2000)) AS v FROM DUAL")
        .bind(payload.clone())
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: Vec<u8> = row.get("V");
    assert_eq!(v, payload);
}

#[tokio::test]
async fn chrono_types() {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
    let mut conn = new_conn().await;

    let dt = NaiveDateTime::parse_from_str("2026-05-16 12:34:56.123456", "%Y-%m-%d %H:%M:%S%.f")
        .unwrap();
    let row = sqlx::query("SELECT CAST(? AS TIMESTAMP) AS v FROM DUAL")
        .bind(dt)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: NaiveDateTime = row.get("V");
    assert_eq!(v, dt);

    let d = NaiveDate::from_ymd_opt(2026, 5, 16).unwrap();
    let row = sqlx::query("SELECT CAST(? AS DATE) AS v FROM DUAL")
        .bind(d)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: NaiveDate = row.get("V");
    assert_eq!(v, d);

    let t = NaiveTime::from_hms_opt(12, 34, 56).unwrap();
    let row = sqlx::query("SELECT CAST(? AS DATE) AS v FROM DUAL")
        .bind(t)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: NaiveTime = row.get("V");
    assert_eq!(v, t);

    let utc_dt = Utc.with_ymd_and_hms(2026, 5, 16, 12, 34, 56).unwrap();
    let row = sqlx::query("SELECT CAST(? AS TIMESTAMP WITH TIME ZONE) AS v FROM DUAL")
        .bind(utc_dt)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: chrono::DateTime<Utc> = row.get("V");
    assert_eq!(v, utc_dt);
}

#[tokio::test]
async fn null_binding() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT ? AS v FROM DUAL")
        .bind(None::<String>)
        .fetch_one(&mut conn)
        .await
        .unwrap();
    let v: Option<String> = row.get("V");
    assert!(v.is_none());
}

#[tokio::test]
async fn pool_basic() {
    use sqlx::pool::PoolOptions;

    let pool = PoolOptions::<sqlx_oracle::Oracle>::new()
        .max_connections(3)
        .test_before_acquire(true)
        .connect(&db_url())
        .await
        .unwrap();

    let row = sqlx::query("SELECT 1 AS n FROM DUAL")
        .fetch_one(&pool)
        .await
        .unwrap();
    let n: i64 = row.get("N");
    assert_eq!(n, 1);

    let mut pool_conn = pool.acquire().await.unwrap();
    let row = sqlx::query("SELECT ? + ? AS sum FROM DUAL")
        .bind(2i64)
        .bind(3i64)
        .fetch_one(&mut *pool_conn)
        .await
        .unwrap();
    let sum: i64 = row.get("SUM");
    assert_eq!(sum, 5);
    drop(pool_conn);

    assert_eq!(pool.size(), 2);

    pool.close().await;
}
