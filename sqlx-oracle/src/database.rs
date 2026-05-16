use crate::arguments::OracleArgumentBuffer;
use crate::value::{OracleValue, OracleValueRef};
use crate::{
    OracleArguments, OracleColumn, OracleConnection, OracleQueryResult, OracleRow,
    OracleStatement, OracleTransactionManager, OracleTypeInfo,
};

pub(crate) use sqlx_core::database::{Database, HasStatementCache};

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

    const NAME: &'static str = "Oracle";

    const URL_SCHEMES: &'static [&'static str] = &["oracle"];
}

impl HasStatementCache for Oracle {}
