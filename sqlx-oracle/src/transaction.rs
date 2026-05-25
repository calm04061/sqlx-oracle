use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};
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
        statement: Option<SqlStr>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send + 'conn {
        async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                let savepoint = AssertSqlSafe(format!("SAVEPOINT _sqlx_savepoint_{depth}")).into_sql_str();
                conn.execute(savepoint).await?;
            } else if let Some(stmt) = statement {
                conn.execute(stmt).await?;
            }
            conn.transaction_depth += 1;
            Ok(())
        }
    }

    fn commit(conn: &mut OracleConnection) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                if depth == 1 {
                    conn.execute("COMMIT").await?;
                } else {
                    let savepoint = AssertSqlSafe(format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1)).into_sql_str();
                    conn.execute(savepoint).await?;
                }
                conn.transaction_depth -= 1;
            }
            Ok(())
        }
    }

    fn rollback(conn: &mut OracleConnection) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        async move {
            let depth = conn.transaction_depth;
            if depth > 0 {
                if depth == 1 {
                    conn.execute("ROLLBACK").await?;
                } else {
                    let savepoint = AssertSqlSafe(format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", depth - 1)).into_sql_str();
                    conn.execute(savepoint).await?;
                }
                conn.transaction_depth -= 1;
            }
            Ok(())
        }
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
