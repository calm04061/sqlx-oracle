use std::borrow::Cow;
use std::fmt::{self, Debug};

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{FutureExt, StreamExt, TryStreamExt};
use once_cell::sync::OnceCell;
use url::Url;

use sqlx_core::connection::{Connection, LogSettings};
use sqlx_core::error::Error;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::Either;
use sqlx_core::arguments::Arguments;
use sqlx_core::describe::Describe;
use sqlx_core::logger::QueryLogger;
use sqlx_core::transaction::Transaction;

use crate::arguments::OracleBindValue;
use crate::error::OracleDbError;
use crate::options::OracleConnectOptions;
use crate::row::OracleRow;
use crate::statement::OracleStatement;
use crate::{
    Oracle, OracleArguments, OracleColumn, OracleQueryResult, OracleTypeInfo, OracleValue,
};

pub(crate) static OCI_ENV: OnceCell<sibyl::Environment> = OnceCell::new();

pub struct OracleConnection {
    session: sibyl::Session<'static>,
    pub(crate) transaction_depth: usize,
    log_settings: LogSettings,
}

impl Debug for OracleConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OracleConnection")
            .field("transaction_depth", &self.transaction_depth)
            .field("log_settings", &self.log_settings)
            .finish()
    }
}

impl OracleConnection {
    pub(crate) async fn establish(
        url: &str,
        log_settings: LogSettings,
    ) -> Result<Self, Error> {
        let parsed = Url::parse(url).map_err(|e| {
            Error::protocol(format!("invalid database URL: {e}"))
        })?;

        let username = {
            let user = parsed.username();
            urlencoding::decode(user)
                .map_err(|e| Error::protocol(format!("invalid username: {e}")))?
                .into_owned()
        };

        let password = parsed
            .password()
            .map(|p| {
                urlencoding::decode(p)
                    .map_err(|e| Error::protocol(format!("invalid password: {e}")))
                    .map(|s| s.into_owned())
            })
            .transpose()?
            .unwrap_or_default();

        let host = parsed.host_str().unwrap_or("localhost");
        let port = parsed.port().unwrap_or(1521);

        let service = parsed.path().trim_start_matches('/');

        let dbname = if !service.is_empty() {
            format!("{host}:{port}/{service}")
        } else {
            format!("{host}:{port}")
        };

        let env = OCI_ENV
            .get_or_try_init(|| sibyl::env().map_err(|e| {
                Error::protocol(format!("failed to create OCI environment: {e}"))
            }))?;

        let session = env
            .connect(&dbname, &username, &password)
            .await
            .map_err(|e| {
                Error::from(OracleDbError::new(format!("failed to connect: {e}")))
            })?;

        Ok(Self {
            session,
            transaction_depth: 0,
            log_settings,
        })
    }

    fn sql_is_query(sql: &str) -> bool {
        let sql = sql.trim();
        if sql.is_empty() {
            return false;
        }
        let first = sql
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_uppercase();
        first == "SELECT" || first == "WITH"
    }

    fn convert_placeholders(sql: &str, num_params: usize) -> String {
        if num_params == 0 {
            return sql.to_owned();
        }

        let mut result = String::with_capacity(sql.len());
        let mut param_index: usize = 1;
        let mut chars = sql.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\'' => {
                    result.push(ch);
                    while let Some(&next) = chars.peek() {
                        result.push(next);
                        chars.next();
                        if next == '\'' {
                            break;
                        }
                    }
                }
                '?' if param_index <= num_params => {
                    use std::fmt::Write;
                    write!(result, ":{}", param_index).unwrap();
                    param_index += 1;
                }
                other => {
                    result.push(other);
                }
            }
        }

        result
    }

    async fn execute_or_query(
        &mut self,
        sql: &str,
        arguments: Option<&mut OracleArguments>,
    ) -> Result<
        impl futures_core::Stream<
            Item = Result<Either<OracleQueryResult, OracleRow>, Error>,
        > + use<'_>,
        Error,
    > {
        let mut logger = QueryLogger::new(sql, self.log_settings.clone());

        let num_params = arguments.as_ref().map(|a| a.len()).unwrap_or(0);
        let oracle_sql = Self::convert_placeholders(sql, num_params);
        let is_query = Self::sql_is_query(sql);

        let mut owned_args = arguments
            .map(|args| build_sibyl_args(args))
            .unwrap_or_default();

        let stmt = self.session.prepare(&oracle_sql).await.map_err(|e| {
            Error::from(OracleDbError::new(format!("prepare failed: {e}")))
        })?;

        if is_query {
            let num_cols = stmt.column_count().map_err(|e| {
                Error::from(OracleDbError::new(format!("get column count failed: {e}")))
            })?;

            let mut columns = Vec::with_capacity(num_cols);
            for i in 0..num_cols {
                let col_info = stmt.column(i).ok_or_else(|| {
                    Error::protocol(format!("column {i} not found"))
                })?;
                let col_name = col_info.name().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column name failed: {e}")))
                })?;
                let col_type = col_info.data_type().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column type failed: {e}")))
                })?;

                columns.push(OracleColumn {
                    ordinal: i,
                    name: col_name.to_owned(),
                    type_info: oracle_type_from_sibyl(col_type),
                });
            }

            let rows = run_query(&stmt, &mut owned_args).await?;

            let mut collected: Vec<Either<OracleQueryResult, OracleRow>> = Vec::new();
            loop {
                match rows.next().await {
                    Ok(Some(sibyl_row)) => {
                        let mut values = Vec::with_capacity(num_cols);

                        for i in 0..num_cols {
                            let is_null = sibyl_row.is_null(i);
                            if is_null {
                                values.push(OracleValue {
                                    value: None,
                                    type_info: columns[i].type_info.clone(),
                                });
                            } else {
                                let text: String = sibyl_row.get(i).map_err(|e| {
                                    Error::Decode(
                                        format!("failed to get column {i}: {e}").into(),
                                    )
                                })?;
                                values.push(OracleValue {
                                    value: Some(text.into_bytes()),
                                    type_info: columns[i].type_info.clone(),
                                });
                            }
                        }

                        let oracle_row = OracleRow {
                            columns: columns.clone(),
                            values,
                        };

                        logger.increment_rows_returned();
                        collected.push(Either::Right(oracle_row));
                    }
                    Ok(None) => break,
                    Err(e) => {
                        return Err(Error::from(OracleDbError::new(format!(
                            "fetch failed: {e}"
                        ))));
                    }
                }
            }

            Ok(try_stream! {
                for item in collected {
                    r#yield!(item);
                }
                Ok(())
            })
        } else {
            let affected = run_execute(&stmt, &mut owned_args).await?;

            let result = OracleQueryResult {
                rows_affected: affected as u64,
            };
            logger.increase_rows_affected(result.rows_affected);

            Ok(try_stream! {
                r#yield!(Either::Left(result));
                Ok(())
            })
        }
    }
}

impl Connection for OracleConnection {
    type Database = Oracle;
    type Options = OracleConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self, None)
    }

    fn begin_with(
        &mut self,
        statement: impl Into<Cow<'static, str>>,
    ) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self, Some(statement.into()))
    }

    fn shrink_buffers(&mut self) {}

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn should_flush(&self) -> bool {
        false
    }
}

impl<'c> Executor<'c> for &'c mut OracleConnection {
    type Database = Oracle;

    fn fetch_many<'e, 'q, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<OracleQueryResult, OracleRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();

        Box::pin(
            async move {
                let mut arguments = match arguments {
                    Ok(Some(args)) => args,
                    Ok(None) => OracleArguments::default(),
                    Err(e) => {
                        return Err(Error::protocol(format!(
                            "error taking arguments: {e}"
                        )));
                    }
                };
                let stream = self
                    .execute_or_query(sql, Some(&mut arguments))
                    .await?;
                Ok(stream.boxed())
            }
            .into_stream(),
        )
        .try_flatten()
        .boxed()
    }

    fn fetch_optional<'e, 'q, E>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<OracleRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let sql = query.sql();
        let arguments = query.take_arguments();

        Box::pin(async move {
            let mut arguments = match arguments {
                Ok(Some(args)) => args,
                Ok(None) => OracleArguments::default(),
                Err(e) => {
                    return Err(Error::protocol(format!(
                        "error taking arguments: {e}"
                    )));
                }
            };
            let stream = self
                .execute_or_query(sql, Some(&mut arguments))
                .await?;

            futures_util::pin_mut!(stream);

            let mut result = None;
            while let Some(item) = stream.try_next().await? {
                if let Either::Right(row) = item {
                    if result.is_none() {
                        result = Some(row);
                    }
                }
            }
            Ok(result)
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _parameters: &'e [OracleTypeInfo],
    ) -> BoxFuture<'e, Result<OracleStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let num_params = _parameters.len();
            let oracle_sql = if num_params > 0 {
                OracleConnection::convert_placeholders(sql, num_params)
            } else {
                sql.to_owned()
            };

            let stmt = self.session.prepare(&oracle_sql).await.map_err(|e| {
                Error::from(OracleDbError::new(format!("prepare failed: {e}")))
            })?;

            let num_cols = stmt.column_count().map_err(|e| {
                Error::from(OracleDbError::new(format!("get column count failed: {e}")))
            })?;

            let mut columns = Vec::with_capacity(num_cols as usize);
            for i in 0..num_cols {
                let col_info = stmt.column(i).ok_or_else(|| {
                    Error::protocol(format!("column {i} not found"))
                })?;
                let col_name = col_info.name().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column name failed: {e}")))
                })?;
                let col_type = col_info.data_type().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column type failed: {e}")))
                })?;

                columns.push(OracleColumn {
                    ordinal: i,
                    name: col_name.to_owned(),
                    type_info: oracle_type_from_sibyl(col_type),
                });
            }

            Ok(OracleStatement {
                sql: Cow::Owned(oracle_sql),
                columns,
            })
        })
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let stmt = self.session.prepare(sql).await.map_err(|e| {
                Error::from(OracleDbError::new(format!("describe prepare failed: {e}")))
            })?;

            let num_cols = stmt.column_count().map_err(|e| {
                Error::from(OracleDbError::new(format!("get column count failed: {e}")))
            })?;

            let mut columns = Vec::with_capacity(num_cols as usize);
            for i in 0..num_cols {
                let col_info = stmt.column(i).ok_or_else(|| {
                    Error::protocol(format!("column {i} not found"))
                })?;
                let col_name = col_info.name().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column name failed: {e}")))
                })?;
                let col_type = col_info.data_type().map_err(|e| {
                    Error::from(OracleDbError::new(format!("get column type failed: {e}")))
                })?;

                columns.push(OracleColumn {
                    ordinal: i,
                    name: col_name.to_owned(),
                    type_info: oracle_type_from_sibyl(col_type),
                });
            }

            Ok(Describe {
                columns,
                nullable: vec![],
                parameters: None,
            })
        })
    }
}

fn oracle_type_from_sibyl(col_type: sibyl::ColumnType) -> OracleTypeInfo {
    use sibyl::ColumnType as SCT;
    match col_type {
        SCT::Char => OracleTypeInfo::Char,
        SCT::NChar => OracleTypeInfo::NChar,
        SCT::Varchar => OracleTypeInfo::Varchar2,
        SCT::NVarchar => OracleTypeInfo::NVarchar2,
        SCT::Clob => OracleTypeInfo::Clob,
        SCT::NClob => OracleTypeInfo::NClob,
        SCT::Long => OracleTypeInfo::Long,
        SCT::Raw => OracleTypeInfo::Raw,
        SCT::LongRaw => OracleTypeInfo::LongRaw,
        SCT::Blob => OracleTypeInfo::Blob,
        SCT::Number => OracleTypeInfo::Number,
        SCT::BinaryFloat => OracleTypeInfo::BinaryFloat,
        SCT::BinaryDouble => OracleTypeInfo::BinaryDouble,
        SCT::Date => OracleTypeInfo::Date,
        SCT::Timestamp => OracleTypeInfo::Timestamp,
        SCT::TimestampWithTimeZone => OracleTypeInfo::TimestampTZ,
        SCT::TimestampWithLocalTimeZone => OracleTypeInfo::TimestampLTZ,
        SCT::IntervalYearToMonth => OracleTypeInfo::IntervalYM,
        SCT::IntervalDayToSecond => OracleTypeInfo::IntervalDS,
        SCT::RowID => OracleTypeInfo::RowID,
        SCT::Cursor => OracleTypeInfo::Unknown("SYS_REFCURSOR".to_owned()),
        SCT::Unknown => OracleTypeInfo::Null,
    }
}

fn build_sibyl_args(args: &mut OracleArguments) -> Vec<Box<dyn sibyl::ToSql>> {
    args.buffer
        .values
        .iter()
        .map(|v| match v {
            OracleBindValue::Null => Box::new(None::<&str>) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Int(i) => Box::new(*i) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Float(f) => Box::new(*f) as Box<dyn sibyl::ToSql>,
            OracleBindValue::String(s) => Box::new(s.clone()) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Bool(b) => Box::new(*b) as Box<dyn sibyl::ToSql>,
        })
        .collect()
}

async fn run_query<'a>(
    stmt: &'a sibyl::Statement<'a>,
    owned_args: &mut Vec<Box<dyn sibyl::ToSql>>,
) -> Result<sibyl::Rows<'a>, Error> {
    let taken = std::mem::take(owned_args);
    let slice: &'static mut [Box<dyn sibyl::ToSql>] = Box::leak(taken.into_boxed_slice());
    let mut refs: Vec<&mut dyn sibyl::ToSql> = slice.iter_mut().map(|b| &mut **b).collect();
    stmt.query(&mut refs).await
        .map_err(|e| Error::from(OracleDbError::new(format!("query failed: {e}"))))
}

async fn run_execute<'a>(
    stmt: &'a sibyl::Statement<'a>,
    owned_args: &mut Vec<Box<dyn sibyl::ToSql>>,
) -> Result<usize, Error> {
    let taken = std::mem::take(owned_args);
    let slice: &'static mut [Box<dyn sibyl::ToSql>] = Box::leak(taken.into_boxed_slice());
    let mut refs: Vec<&mut dyn sibyl::ToSql> = slice.iter_mut().map(|b| &mut **b).collect();
    stmt.execute(&mut refs).await
        .map_err(|e| Error::from(OracleDbError::new(format!("execute failed: {e}"))))
}
