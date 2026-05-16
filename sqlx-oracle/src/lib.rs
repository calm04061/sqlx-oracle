//! `sqlx-oracle` —— 基于 `sibyl` (Oracle OCI 绑定) 的 sqlx 数据库驱动。
//!
//! 实现了 `sqlx::Database` trait，提供 Oracle 数据库的异步连接、
//! 查询执行、事务管理、类型编解码等功能。
//!
//! # 连接 URL 格式
//! ```text
//! oracle://user:password@host:port/service_name
//! ```
//!
//! # 示例
//! ```rust,no_run
//! use sqlx::Connection;
//! use sqlx_oracle::OracleConnection;
//!
//! let mut conn = OracleConnection::connect("oracle://scott:tiger@localhost:1521/FREEPDB1").await.unwrap();
//! let row = sqlx::query("SELECT 1 FROM DUAL").fetch_one(&mut conn).await.unwrap();
//! ```

#![deny(unsafe_code)]

#[macro_use]
extern crate sqlx_core;

// ---------------------------------------------------------------------------
// 模块声明：每个模块对应 sqlx 框架中的一个概念
// ---------------------------------------------------------------------------
pub mod type_info;
pub mod query_result;
pub mod column;
pub mod value;
pub mod error;
pub mod arguments;
pub mod encode_decode;
pub mod row;
pub mod statement;
pub mod options;
pub mod connection;
pub mod transaction;
pub mod database;

// ---------------------------------------------------------------------------
// 公开 API 重导出
// ---------------------------------------------------------------------------
pub use type_info::OracleTypeInfo;
pub use query_result::OracleQueryResult;
pub use column::OracleColumn;
pub use value::{OracleValue, OracleValueRef};
pub use error::OracleDbError;
pub use arguments::{OracleArguments, OracleArgumentBuffer, OracleBindValue};
pub use row::OracleRow;
pub use statement::OracleStatement;
pub use connection::OracleConnection;
pub use options::OracleConnectOptions;
pub use transaction::OracleTransactionManager;
pub use database::Oracle;
