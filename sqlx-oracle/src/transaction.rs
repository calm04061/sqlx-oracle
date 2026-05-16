use std::borrow::Cow;

use futures_core::future::BoxFuture;
use sqlx_core::transaction::TransactionManager;

use crate::{OracleConnection, Oracle};
use sqlx_core::error::Error;
use sqlx_core::executor::Executor;

/// Oracle 事务管理器。
///
/// 通过 ANSI 标准 SQL (`BEGIN` / `COMMIT` / `ROLLBACK`) 管理事务，
/// 利用 `transaction_depth` 支持嵌套事务（保存点）。
pub struct OracleTransactionManager;

impl TransactionManager for OracleTransactionManager {
    type Database = Oracle;

    fn begin<'conn>(
        conn: &'conn mut OracleConnection,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.transaction_depth;
            let sql = match statement {
                Some(_) if depth > 0 => return Err(Error::InvalidSavePointStatement),
                Some(stmt) => stmt,
                None => sqlx_core::transaction::begin_ansi_transaction_sql(depth),
            };
            conn.execute(&*sql).await?;
            conn.transaction_depth += 1;
            Ok(())
        })
    }

    fn commit(conn: &mut OracleConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.execute(
                    &*sqlx_core::transaction::commit_ansi_transaction_sql(conn.transaction_depth),
                )
                .await?;
                conn.transaction_depth -= 1;
            }
            Ok(())
        })
    }

    fn rollback(conn: &mut OracleConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.execute(
                    &*sqlx_core::transaction::rollback_ansi_transaction_sql(conn.transaction_depth),
                )
                .await?;
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
