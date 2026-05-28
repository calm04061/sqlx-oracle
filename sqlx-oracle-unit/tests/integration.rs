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

// ---------------------------------------------------------------------------
// 复杂查询测试：多表 JOIN
// ---------------------------------------------------------------------------

#[tokio::test]
async fn join_multi_table() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_j_emp").execute(&mut conn).await;
    let _ = sqlx::query("DROP TABLE sqlx_test_j_dept").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_j_dept (deptno NUMBER, dname VARCHAR2(50)) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("CREATE TABLE sqlx_test_j_emp (empno NUMBER, ename VARCHAR2(50), deptno NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    sqlx::query("INSERT INTO sqlx_test_j_dept (deptno, dname) VALUES (?, ?)")
        .bind(10i64).bind("ACCOUNTING").execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_dept (deptno, dname) VALUES (?, ?)")
        .bind(20i64).bind("RESEARCH").execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp (empno, ename, deptno) VALUES (?, ?, ?)")
        .bind(101i64).bind("Alice").bind(10i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp (empno, ename, deptno) VALUES (?, ?, ?)")
        .bind(102i64).bind("Bob").bind(20i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp (empno, ename, deptno) VALUES (?, ?, ?)")
        .bind(103i64).bind("Carol").bind(10i64).execute(&mut conn).await.unwrap();

    let rows = sqlx::query(
        "SELECT e.empno, e.ename, d.dname \
         FROM sqlx_test_j_emp e JOIN sqlx_test_j_dept d ON e.deptno = d.deptno \
         ORDER BY e.empno"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 3);

    let empno: i64 = rows[0].get("EMPNO");
    let ename: String = rows[0].get("ENAME");
    let dname: String = rows[0].get("DNAME");
    assert_eq!(empno, 101);
    assert_eq!(ename, "Alice");
    assert_eq!(dname, "ACCOUNTING");

    sqlx::query("DROP TABLE sqlx_test_j_emp").execute(&mut conn).await.unwrap();
    sqlx::query("DROP TABLE sqlx_test_j_dept").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：GROUP BY + HAVING + 聚合
// ---------------------------------------------------------------------------

#[tokio::test]
async fn group_by_having() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_gb").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_gb (cat VARCHAR2(20), val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    for (cat, val) in [("A", 10), ("A", 20), ("B", 30), ("B", 40), ("C", 50)] {
        sqlx::query("INSERT INTO sqlx_test_gb (cat, val) VALUES (?, ?)")
            .bind(cat).bind(val as i64)
            .execute(&mut conn).await.unwrap();
    }

    let rows = sqlx::query(
        "SELECT cat, COUNT(*) AS cnt, SUM(val) AS total \
         FROM sqlx_test_gb GROUP BY cat HAVING SUM(val) > ? ORDER BY cat"
    ).bind(25i64).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 2);
    let cat0: String = rows[0].get("CAT");
    let cnt0: i64 = rows[0].get("CNT");
    let total0: i64 = rows[0].get("TOTAL");
    assert_eq!(cat0, "A");
    assert_eq!(cnt0, 2);
    assert_eq!(total0, 30);
    let cat1: String = rows[1].get("CAT");
    let total1: i64 = rows[1].get("TOTAL");
    assert_eq!(cat1, "B");
    assert_eq!(total1, 70);

    sqlx::query("DROP TABLE sqlx_test_gb").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：子查询
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subquery_in_where() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_sub").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_sub (id NUMBER, score NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    for (id, score) in [(1, 80), (2, 90), (3, 75), (4, 95)] {
        sqlx::query("INSERT INTO sqlx_test_sub (id, score) VALUES (?, ?)")
            .bind(id as i64).bind(score as i64)
            .execute(&mut conn).await.unwrap();
    }

    let row = sqlx::query(
        "SELECT id, score FROM sqlx_test_sub \
         WHERE score > (SELECT AVG(score) FROM sqlx_test_sub) ORDER BY id"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(row.len(), 2);
    let ids: Vec<i64> = row.iter().map(|r| r.get("ID")).collect();
    assert_eq!(ids, vec![2, 4]);

    sqlx::query("DROP TABLE sqlx_test_sub").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：WITH (CTE)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cte_with() {
    let mut conn = new_conn().await;
    let row = sqlx::query(
        "WITH numbers AS (SELECT LEVEL AS n FROM DUAL CONNECT BY LEVEL <= ?) \
         SELECT SUM(n) AS total, AVG(n) AS avg FROM numbers"
    ).bind(10i64).fetch_one(&mut conn).await.unwrap();
    let total: i64 = row.get("TOTAL");
    let avg: f64 = row.get("AVG");
    assert_eq!(total, 55);
    assert!((avg - 5.5).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// 复杂查询测试：CASE 表达式
// ---------------------------------------------------------------------------

#[tokio::test]
async fn case_expression() {
    let mut conn = new_conn().await;
    let row = sqlx::query(
        "SELECT CASE ? WHEN 1 THEN 'one' WHEN 2 THEN 'two' ELSE 'other' END AS v FROM DUAL"
    ).bind(2i64).fetch_one(&mut conn).await.unwrap();
    let v: String = row.get("V");
    assert_eq!(v, "two");

    let row = sqlx::query(
        "SELECT CASE WHEN ? > 0 THEN 'positive' WHEN ? < 0 THEN 'negative' ELSE 'zero' END AS v FROM DUAL"
    ).bind(0i64).bind(0i64).fetch_one(&mut conn).await.unwrap();
    let v: String = row.get("V");
    assert_eq!(v, "zero");
}

// ---------------------------------------------------------------------------
// 复杂查询测试：窗口函数
// ---------------------------------------------------------------------------

#[tokio::test]
async fn window_function() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_win").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_win (grp VARCHAR2(10), val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    for (grp, val) in [("A", 30), ("A", 10), ("A", 20), ("B", 5), ("B", 15)] {
        sqlx::query("INSERT INTO sqlx_test_win (grp, val) VALUES (?, ?)")
            .bind(grp).bind(val as i64)
            .execute(&mut conn).await.unwrap();
    }

    let rows = sqlx::query(
        "SELECT grp, val, \
         ROW_NUMBER() OVER (PARTITION BY grp ORDER BY val) AS rn, \
         RANK() OVER (PARTITION BY grp ORDER BY val) AS rk, \
         DENSE_RANK() OVER (PARTITION BY grp ORDER BY val) AS dr \
         FROM sqlx_test_win ORDER BY grp, val"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 5);

    assert_eq!(rows[0].get::<String, &str>("GRP"), "A");
    assert_eq!(rows[0].get::<i64, &str>("VAL"), 10);
    assert_eq!(rows[0].get::<i64, &str>("RN"), 1);

    assert_eq!(rows[2].get::<String, &str>("GRP"), "A");
    assert_eq!(rows[2].get::<i64, &str>("VAL"), 30);
    assert_eq!(rows[2].get::<i64, &str>("RN"), 3);

    sqlx::query("DROP TABLE sqlx_test_win").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：大量参数绑定
// ---------------------------------------------------------------------------

#[tokio::test]
async fn many_parameters() {
    let mut conn = new_conn().await;
    let rows = sqlx::query(
        "SELECT ? + ? + ? + ? + ? + ? + ? + ? + ? + ? AS sum FROM DUAL"
    )
    .bind(1i64).bind(2i64).bind(3i64).bind(4i64).bind(5i64)
    .bind(6i64).bind(7i64).bind(8i64).bind(9i64).bind(10i64)
    .fetch_one(&mut conn).await.unwrap();
    let sum: i64 = rows.get("SUM");
    assert_eq!(sum, 55);
}

// ---------------------------------------------------------------------------
// 复杂查询测试：混合类型绑定
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mixed_type_bind() {
    let mut conn = new_conn().await;
    let row = sqlx::query(
        "SELECT ? AS n, ? AS f, ? AS s, ? AS b FROM DUAL"
    )
    .bind(42i64)
    .bind(3.14f64)
    .bind("hello")
    .bind(true)
    .fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<i64, &str>("N"), 42);
    assert!((row.get::<f64, &str>("F") - 3.14).abs() < 1e-10);
    assert_eq!(row.get::<String, &str>("S"), "hello");
    assert_eq!(row.get::<bool, &str>("B"), true);
}

// ---------------------------------------------------------------------------
// 复杂查询测试：NULL 排序
// ---------------------------------------------------------------------------

#[tokio::test]
async fn null_ordering() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_null").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_null (id NUMBER, val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    sqlx::query("INSERT INTO sqlx_test_null VALUES (?, ?)").bind(1i64).bind(30i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_null VALUES (?, ?)").bind(2i64).bind(None::<i64>).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_null VALUES (?, ?)").bind(3i64).bind(10i64).execute(&mut conn).await.unwrap();

    let rows = sqlx::query(
        "SELECT id FROM sqlx_test_null ORDER BY val NULLS FIRST"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get::<i64, &str>("ID"), 2);
    assert_eq!(rows[1].get::<i64, &str>("ID"), 3);
    assert_eq!(rows[2].get::<i64, &str>("ID"), 1);

    sqlx::query("DROP TABLE sqlx_test_null").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：DISTINCT + UNION
// ---------------------------------------------------------------------------

#[tokio::test]
async fn distinct_union() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_du").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_du (x NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    for v in [1i64, 2, 1, 3, 2] {
        sqlx::query("INSERT INTO sqlx_test_du VALUES (?)").bind(v).execute(&mut conn).await.unwrap();
    }

    let rows = sqlx::query(
        "SELECT x, COUNT(*) AS cnt FROM sqlx_test_du GROUP BY x \
         UNION \
         SELECT -1, 0 FROM DUAL ORDER BY x"
    ).fetch_all(&mut conn).await.unwrap();
    // Results: -1 (0), 1 (2), 2 (2), 3 (1)
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].get::<i64, &str>("X"), -1);
    assert_eq!(rows[3].get::<i64, &str>("X"), 3);
    assert_eq!(rows[3].get::<i64, &str>("CNT"), 1);

    sqlx::query("DROP TABLE sqlx_test_du").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：MERGE (upsert)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn merge_upsert() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_mrg").execute(&mut conn).await;
    let _ = sqlx::query("DROP TABLE sqlx_test_mrg_src").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_mrg (id NUMBER PRIMARY KEY, val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("CREATE TABLE sqlx_test_mrg_src (id NUMBER, val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    sqlx::query("INSERT INTO sqlx_test_mrg VALUES (?, ?)").bind(1i64).bind(10i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_mrg VALUES (?, ?)").bind(2i64).bind(20i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_mrg_src VALUES (?, ?)").bind(1i64).bind(100i64).execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_mrg_src VALUES (?, ?)").bind(3i64).bind(300i64).execute(&mut conn).await.unwrap();

    let affected = sqlx::query(
        "MERGE INTO sqlx_test_mrg t \
         USING sqlx_test_mrg_src s ON (t.id = s.id) \
         WHEN MATCHED THEN UPDATE SET t.val = s.val \
         WHEN NOT MATCHED THEN INSERT (id, val) VALUES (s.id, s.val)"
    ).execute(&mut conn).await.unwrap().rows_affected();
    assert_eq!(affected, 2);

    let rows = sqlx::query("SELECT id, val FROM sqlx_test_mrg ORDER BY id")
        .fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].get::<i64, &str>("VAL"), 100);
    assert_eq!(rows[1].get::<i64, &str>("VAL"), 20);
    assert_eq!(rows[2].get::<i64, &str>("VAL"), 300);

    sqlx::query("DROP TABLE sqlx_test_mrg_src").execute(&mut conn).await.unwrap();
    sqlx::query("DROP TABLE sqlx_test_mrg").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：日期计算
// ---------------------------------------------------------------------------

#[tokio::test]
async fn date_arithmetic() {
    use chrono::NaiveDate;
    let mut conn = new_conn().await;
    let row = sqlx::query(
        "SELECT TO_DATE(?, 'YYYY-MM-DD') + ? AS future FROM DUAL"
    ).bind("2026-01-01").bind(30i64)
        .fetch_one(&mut conn).await.unwrap();
    let future: NaiveDate = row.get("FUTURE");
    assert_eq!(future, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
}

// ---------------------------------------------------------------------------
// 复杂查询测试：LIKE 模式匹配
// ---------------------------------------------------------------------------

#[tokio::test]
async fn like_pattern() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_like").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_like (name VARCHAR2(50)) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    for name in ["Alice", "Bob", "Charlie", "Dave"] {
        sqlx::query("INSERT INTO sqlx_test_like VALUES (?)")
            .bind(name).execute(&mut conn).await.unwrap();
    }

    let rows = sqlx::query(
        "SELECT name FROM sqlx_test_like WHERE name LIKE ? ORDER BY name"
    ).bind("A%").fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String, &str>("NAME"), "Alice");

    let rows = sqlx::query(
        "SELECT name FROM sqlx_test_like WHERE name LIKE ? ORDER BY name"
    ).bind("%e").fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].get::<String, &str>("NAME"), "Alice");
    assert_eq!(rows[1].get::<String, &str>("NAME"), "Charlie");

    sqlx::query("DROP TABLE sqlx_test_like").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：IN 列表
// ---------------------------------------------------------------------------

#[tokio::test]
async fn in_list() {
    let mut conn = new_conn().await;
    let rows = sqlx::query(
        "SELECT ? AS v FROM DUAL WHERE ? IN (?, ?, ?)"
    ).bind(3i64).bind(3i64).bind(1i64).bind(2i64).bind(3i64)
        .fetch_optional(&mut conn).await.unwrap();
    assert!(rows.is_some());
    assert_eq!(rows.unwrap().get::<i64, &str>("V"), 3);

    let rows = sqlx::query(
        "SELECT ? AS v FROM DUAL WHERE ? NOT IN (?, ?, ?)"
    ).bind(99i64).bind(99i64).bind(1i64).bind(2i64).bind(3i64)
        .fetch_optional(&mut conn).await.unwrap();
    assert!(rows.is_some());
}

// ---------------------------------------------------------------------------
// 复杂查询测试：EXISTS
// ---------------------------------------------------------------------------

#[tokio::test]
async fn exists_subquery() {
    let mut conn = new_conn().await;
    let _ = sqlx::query("DROP TABLE sqlx_test_ex").execute(&mut conn).await;

    sqlx::query("CREATE TABLE sqlx_test_ex (id NUMBER, active VARCHAR2(1)) NOPARALLEL")
        .execute(&mut conn).await.unwrap();

    sqlx::query("INSERT INTO sqlx_test_ex VALUES (?, ?)").bind(1i64).bind("Y").execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_ex VALUES (?, ?)").bind(2i64).bind("N").execute(&mut conn).await.unwrap();

    let row = sqlx::query(
        "SELECT ? AS v FROM DUAL \
         WHERE EXISTS (SELECT 1 FROM sqlx_test_ex WHERE active = ?)"
    ).bind(1i64).bind("Y").fetch_optional(&mut conn).await.unwrap();
    assert!(row.is_some());

    let row = sqlx::query(
        "SELECT ? AS v FROM DUAL \
         WHERE EXISTS (SELECT 1 FROM sqlx_test_ex WHERE active = ?)"
    ).bind(1i64).bind("X").fetch_optional(&mut conn).await.unwrap();
    assert!(row.is_none());

    sqlx::query("DROP TABLE sqlx_test_ex").execute(&mut conn).await.unwrap();
}

// ---------------------------------------------------------------------------
// 复杂查询测试：CLOB 字符串
// ---------------------------------------------------------------------------

#[tokio::test]
async fn clob_string() {
    let mut conn = new_conn().await;
    let long_str = "A".repeat(5000);
    let row = sqlx::query("SELECT CAST(? AS CLOB) AS v FROM DUAL")
        .bind(&long_str)
        .fetch_one(&mut conn).await.unwrap();
    let v: String = row.get("V");
    assert_eq!(v.len(), 5000);
    assert_eq!(v, long_str);
}

// ---------------------------------------------------------------------------
// 复杂查询测试：BETWEEN
// ---------------------------------------------------------------------------

#[tokio::test]
async fn between_range() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT ? AS v FROM DUAL WHERE ? BETWEEN ? AND ?")
        .bind(5i64).bind(5i64).bind(1i64).bind(10i64)
        .fetch_optional(&mut conn).await.unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().get::<i64, &str>("V"), 5);

    let row = sqlx::query("SELECT ? AS v FROM DUAL WHERE ? BETWEEN ? AND ?")
        .bind(99i64).bind(99i64).bind(1i64).bind(10i64)
        .fetch_optional(&mut conn).await.unwrap();
    assert!(row.is_none());
}

// ---------------------------------------------------------------------------
// 复杂查询测试：DECODE 函数
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decode_function() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT DECODE(?, 1, 'one', 2, 'two', 'other') AS v FROM DUAL")
        .bind(2i64).fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<String, &str>("V"), "two");

    let row = sqlx::query("SELECT DECODE(?, 1, 'one', 2, 'two', 'other') AS v FROM DUAL")
        .bind(99i64).fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<String, &str>("V"), "other");
}

// ---------------------------------------------------------------------------
// 复杂查询测试：NVL / COALESCE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn nvl_coalesce() {
    let mut conn = new_conn().await;
    let row = sqlx::query("SELECT NVL(?, 'default') AS v FROM DUAL")
        .bind(None::<String>).fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<String, &str>("V"), "default");

    let row = sqlx::query("SELECT COALESCE(?, ?, 'fallback') AS v FROM DUAL")
        .bind(None::<String>).bind(None::<String>)
        .fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<String, &str>("V"), "fallback");

    let row = sqlx::query("SELECT COALESCE(?, 'not null') AS v FROM DUAL")
        .bind("hello").fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.get::<String, &str>("V"), "hello");
}

// ---------------------------------------------------------------------------
// 结构体查询测试：query_as + FromRow
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct DualRow {
    n: i64,
    s: String,
}

#[tokio::test]
async fn struct_from_dual() {
    let mut conn = new_conn().await;
    let row: DualRow = sqlx::query_as(
        "SELECT 42 AS n, 'hello' AS s FROM DUAL"
    ).fetch_one(&mut conn).await.unwrap();
    assert_eq!(row.n, 42);
    assert_eq!(row.s, "hello");
}

#[tokio::test]
async fn struct_query_as_vec() {
    let mut conn = new_conn().await;
    let rows: Vec<DualRow> = sqlx::query_as(
        "SELECT 1 AS n, 'a' AS s FROM DUAL UNION ALL SELECT 2, 'b' FROM DUAL"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].n, 1);
    assert_eq!(rows[0].s, "a");
    assert_eq!(rows[1].n, 2);
    assert_eq!(rows[1].s, "b");
}

#[derive(sqlx::FromRow)]
struct InsertSelectRow {
    id: i64,
    label: String,
}

#[tokio::test]
async fn struct_from_table() {
    let mut conn = new_conn().await;
    sqlx::query("CREATE TABLE sqlx_test_struct (id NUMBER, label VARCHAR2(100)) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_struct VALUES (?, ?)")
        .bind(10i64).bind("alpha").execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_struct VALUES (?, ?)")
        .bind(20i64).bind("beta").execute(&mut conn).await.unwrap();

    let rows: Vec<InsertSelectRow> = sqlx::query_as(
        "SELECT id, label FROM sqlx_test_struct ORDER BY id"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 10);
    assert_eq!(rows[0].label, "alpha");
    assert_eq!(rows[1].id, 20);
    assert_eq!(rows[1].label, "beta");

    sqlx::query("DROP TABLE sqlx_test_struct").execute(&mut conn).await.unwrap();
}

#[derive(sqlx::FromRow)]
struct ChronoRow {
    ts: chrono::NaiveDateTime,
    d: chrono::NaiveDate,
}

#[tokio::test]
async fn struct_with_chrono() {
    use chrono::{NaiveDate, NaiveDateTime};
    let mut conn = new_conn().await;
    let row: ChronoRow = sqlx::query_as(
        "SELECT CAST('2026-06-15 10:30:00' AS TIMESTAMP) AS ts, \
                CAST('2026-06-15' AS DATE) AS d FROM DUAL"
    ).fetch_one(&mut conn).await.unwrap();

    let expected_ts = NaiveDateTime::parse_from_str("2026-06-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let expected_d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
    assert_eq!(row.ts, expected_ts);
    assert_eq!(row.d, expected_d);
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct NullableRow {
    id: i64,
    val: Option<i64>,
}

#[tokio::test]
async fn struct_with_option() {
    let mut conn = new_conn().await;
    sqlx::query("CREATE TABLE sqlx_test_opt (id NUMBER, val NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_opt VALUES (?, ?)").bind(1i64).bind(Some(42i64))
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_opt VALUES (?, ?)").bind(2i64).bind(None::<i64>)
        .execute(&mut conn).await.unwrap();

    let rows: Vec<NullableRow> = sqlx::query_as(
        "SELECT id, val FROM sqlx_test_opt ORDER BY id"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 1);
    assert_eq!(rows[0].val, Some(42));
    assert_eq!(rows[1].id, 2);
    assert_eq!(rows[1].val, None);

    sqlx::query("DROP TABLE sqlx_test_opt").execute(&mut conn).await.unwrap();
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct DeptRow {
    deptno: i64,
    dname: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct EmpJoinRow {
    empno: i64,
    ename: String,
    dname: String,
}

#[tokio::test]
async fn struct_join() {
    let mut conn = new_conn().await;
    sqlx::query("CREATE TABLE sqlx_test_j_dept2 (deptno NUMBER, dname VARCHAR2(50)) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("CREATE TABLE sqlx_test_j_emp2 (empno NUMBER, ename VARCHAR2(50), deptno NUMBER) NOPARALLEL")
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_dept2 VALUES (?, ?)").bind(10i64).bind("IT")
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_dept2 VALUES (?, ?)").bind(20i64).bind("HR")
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp2 VALUES (?, ?, ?)").bind(1i64).bind("Alice").bind(10i64)
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp2 VALUES (?, ?, ?)").bind(2i64).bind("Bob").bind(10i64)
        .execute(&mut conn).await.unwrap();
    sqlx::query("INSERT INTO sqlx_test_j_emp2 VALUES (?, ?, ?)").bind(3i64).bind("Carol").bind(20i64)
        .execute(&mut conn).await.unwrap();

    let rows: Vec<EmpJoinRow> = sqlx::query_as(
        "SELECT e.empno, e.ename, d.dname \
         FROM sqlx_test_j_emp2 e JOIN sqlx_test_j_dept2 d ON e.deptno = d.deptno \
         ORDER BY e.empno"
    ).fetch_all(&mut conn).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].empno, 1);
    assert_eq!(rows[0].ename, "Alice");
    assert_eq!(rows[0].dname, "IT");
    assert_eq!(rows[2].empno, 3);
    assert_eq!(rows[2].ename, "Carol");
    assert_eq!(rows[2].dname, "HR");

    sqlx::query("DROP TABLE sqlx_test_j_emp2").execute(&mut conn).await.unwrap();
    sqlx::query("DROP TABLE sqlx_test_j_dept2").execute(&mut conn).await.unwrap();
}
