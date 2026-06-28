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

    /// 开始事务（或嵌套事务的 savepoint）。
    ///
    /// - 深度 0→1：Oracle 隐式事务已生效，只做 depth += 1，不发任何 SQL。
    /// - 深度 n→n+1：执行 `SAVEPOINT _sqlx_savepoint_{n}`。
    /// - 如果提供了自定义起始语句（来自 `begin_with`），在深度 0→1 时执行该语句。
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

    /// 提交事务（或释放 savepoint）。
    ///
    /// - 深度 1：执行 `COMMIT` 提交顶层事务。
    /// - 深度 >1：执行 `RELEASE SAVEPOINT _sqlx_savepoint_{depth-1}`。
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

    /// 回滚事务（或回滚到 savepoint）。
    ///
    /// - 深度 1：执行 `ROLLBACK` 回滚整个事务。
    /// - 深度 >1：执行 `ROLLBACK TO SAVEPOINT _sqlx_savepoint_{depth-1}`。
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

    /// 标记回滚（仅减深度，不执行 SQL）。
    ///
    /// 当 Drop 时自动回滚事务，释放对应的深度层级。
    fn start_rollback(conn: &mut OracleConnection) {
        if conn.transaction_depth > 0 {
            conn.transaction_depth -= 1;
        }
    }

    /// 返回当前事务嵌套深度。
    fn get_transaction_depth(conn: &OracleConnection) -> usize {
        conn.transaction_depth
    }
}
