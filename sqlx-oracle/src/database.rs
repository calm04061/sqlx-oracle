use crate::arguments::OracleArgumentBuffer;
use crate::value::{OracleValue, OracleValueRef};
use crate::{
    OracleArguments, OracleColumn, OracleConnection, OracleQueryResult, OracleRow,
    OracleStatement, OracleTransactionManager, OracleTypeInfo,
};

pub(crate) use sqlx_core::database::{Database, HasStatementCache};

/// Oracle 数据库类型标识。
///
/// 作为零尺寸类型实现 `sqlx::Database` trait，将所有关联类型
/// 映射到本 crate 中定义的 Oracle 专属实现。
#[derive(Debug)]
pub struct Oracle;

impl Database for Oracle {
    type Connection = OracleConnection;

    type TransactionManager = OracleTransactionManager;

    type Row = OracleRow;

    type QueryResult = OracleQueryResult;

    type Column = OracleColumn;

    type TypeInfo = OracleTypeInfo;

    type Value = OracleValue;
    type ValueRef<'r> = OracleValueRef<'r>;

    type Arguments<'q> = OracleArguments;
    type ArgumentBuffer<'q> = OracleArgumentBuffer;

    type Statement<'q> = OracleStatement<'q>;

    /// 数据库产品名称
    const NAME: &'static str = "Oracle";

    /// 连接 URL 支持 `oracle://` 协议前缀
    const URL_SCHEMES: &'static [&'static str] = &["oracle"];
}

impl HasStatementCache for Oracle {}
