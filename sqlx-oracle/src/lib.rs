#![forbid(unsafe_code)]

#[macro_use]
extern crate sqlx_core;

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
