use std::borrow::Cow;

use futures_core::future::BoxFuture;
use sqlx_core::transaction::TransactionManager;

use crate::{OracleConnection, Oracle};
use sqlx_core::error::Error;
use sqlx_core::executor::Executor;

/// Oracle 事务管理器。
///
/// Oracle 使用隐式事务：第一个 DML 自动开启事务。
/// 对于嵌套事务（savepoint）使用 `SAVEPOINT` / `RELEASE SAVEPOINT` / `ROLLBACK TO SAVEPOINT`。
pub struct OracleTransactionManager;

impl TransactionManager for OracleTransactionManager {
    type Database = Oracle;

    fn begin<'conn>(
        conn: &'conn mut OracleConnection,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                // 嵌套事务：使用 SAVEPOINT
                let savepoint = format!("SAVEPOINT _sqlx_savepoint_{depth}");
                conn.execute(&*savepoint).await?;
            } else if let Some(stmt) = statement {
                // 用户自定义语句（极少使用）
                conn.execute(&*stmt).await?;
            }
            // depth == 0 且无自定义语句：Oracle 隐式事务，无需 SQL
            conn.transaction_depth += 1;
            Ok(())
        })
    }

    fn commit(conn: &mut OracleConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                if depth == 1 {
                    // 最外层事务：COMMIT
                    conn.execute("COMMIT").await?;
                } else {
                    // 释放保存点
                    let savepoint = format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1);
                    conn.execute(&*savepoint).await?;
                }
                conn.transaction_depth -= 1;
            }
            Ok(())
        })
    }

    fn rollback(conn: &mut OracleConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                if depth == 1 {
                    // 最外层事务：ROLLBACK
                    conn.execute("ROLLBACK").await?;
                } else {
                    // 回滚到保存点
                    let savepoint = format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", depth - 1);
                    conn.execute(&*savepoint).await?;
                }
                conn.transaction_depth -= 1;
            }
            Ok(())
        })
    }

    fn start_rollback(conn: &mut OracleConnection) {
        if conn.transaction_depth > 0 {
            conn.transaction_depth -= 1;
        }
    }

    fn get_transaction_depth(conn: &OracleConnection) -> usize {
        conn.transaction_depth
    }
}
