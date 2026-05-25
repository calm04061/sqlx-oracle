use std::fmt::{self, Debug};
use std::path::PathBuf;

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
use sqlx_core::logger::QueryLogger;
use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr, SqlStr};
use sqlx_core::transaction::Transaction;

use crate::arguments::OracleBindValue;
use crate::error::OracleDbError;
use crate::options::OracleConnectOptions;
use crate::row::OracleRow;
use crate::statement::OracleStatement;
use crate::{
    Oracle, OracleArguments, OracleColumn, OracleQueryResult, OracleTypeInfo, OracleValue,
};

/// 全局 OCI 环境（懒初始化）。
pub(crate) static OCI_ENV: OnceCell<sibyl::Environment> = OnceCell::new();

/// Oracle 数据库连接。
///
/// 包装 `sibyl::Session`，提供 sqlx 框架所需的 `Connection` 和 `Executor` trait 实现。
/// 内部处理占位符转换（`?` → `:n`）、OCI 会话管理和类型映射。
pub struct OracleConnection {
    /// sibyl OCI 会话（'static 生命周期由 OCI_ENV 保证）
    session: sibyl::Session<'static>,
    /// 当前事务嵌套深度
    pub(crate) transaction_depth: usize,
    /// 日志设置
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
    /// 建立到 Oracle 数据库的连接。
    ///
    /// # URL 格式
    ///
    /// ## 直连 (ezconnect)
    /// ```text
    /// oracle://user:password@host:port/service_name
    /// ```
    ///
    /// ## TNS 别名（需要 tnsnames.ora）
    /// ```text
    /// oracle://user:password@tns_alias
    /// ```
    ///
    /// ## ATP/TCPS（需要 Oracle wallet 目录）
    /// ```text
    /// oracle://user:password@tns_alias?wallet=/path/to/wallet_dir
    /// ```
    ///
    /// 连接建立后自动设置 NLS 会话参数。
    #[allow(unsafe_code)]
    pub(crate) async fn establish(
        url: &str,
        log_settings: LogSettings,
        wallet_path: Option<PathBuf>,
    ) -> Result<Self, Error> {
        // 解析连接 URL
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

        // 判断是否是 TNS 别名：
        // 如果 host 中没有 '.' 且没有 service 路径，视为 TNS 别名（如 "manager"）
        let is_tns_alias = !host.contains('.') && service.is_empty();

        let dbname = if is_tns_alias {
            // TNS 别名：直接使用 host 部分（如 "manager"），
            // OCI 通过 $TNS_ADMIN/tnsnames.ora 解析
            host.to_owned()
        } else if !service.is_empty() {
            // ezconnect 格式：host:port/service_name
            format!("{host}:{port}/{service}")
        } else {
            // 仅 host:port
            format!("{host}:{port}")
        };

        // 设置 TNS_ADMIN 环境变量（用于 ATP/TCPS 钱包连接）
        // 仅在未设置时写入，避免覆盖外部设置
        if let Some(ref wallet) = wallet_path {
            if std::env::var("TNS_ADMIN").is_err() {
                let canonical = std::fs::canonicalize(wallet)
                    .map_err(|e| Error::protocol(format!("invalid wallet path: {e}")))?;
                // SAFETY: TNS_ADMIN 在 OCI 连接初始化前写入，
                // 之后不再修改，不会造成数据竞争。
                unsafe { std::env::set_var("TNS_ADMIN", &canonical); }
            }
        }

        // 初始化全局 OCI 环境（仅首次调用时创建）
        let env = OCI_ENV
            .get_or_try_init(|| sibyl::env().map_err(|e| {
                Error::protocol(format!("failed to create OCI environment: {e}"))
            }))?;

        // 建立会话
        let session = env
            .connect(&dbname, &username, &password)
            .await
            .map_err(|e| {
                Error::from(OracleDbError::new(format!("failed to connect: {e}")))
            })?;

        // 设置 NLS 参数，确保与 chrono 的 ISO 8601 格式兼容
        {
            let nls_stmt = session.prepare(
                "ALTER SESSION SET NLS_DATE_FORMAT = 'YYYY-MM-DD HH24:MI:SS' \
                 NLS_TIMESTAMP_FORMAT = 'YYYY-MM-DD HH24:MI:SS.FF6' \
                 NLS_TIMESTAMP_TZ_FORMAT = 'YYYY-MM-DD HH24:MI:SS.FF6 TZR'"
            ).await.map_err(|e| {
                Error::from(OracleDbError::new(format!("failed to prepare NLS: {e}")))
            })?;
            nls_stmt.execute(()).await.map_err(|e| {
                Error::from(OracleDbError::new(format!("failed to set NLS: {e}")))
            })?;
        }

        Ok(Self {
            session,
            transaction_depth: 0,
            log_settings,
        })
    }

    /// 判断 SQL 是否为查询（以 SELECT 或 WITH 开头）。
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

    /// 将 SQL 中的 `?` 或 `$N` 占位符转换为 Oracle 的 `:n` 命名参数格式。
    ///
    /// 跳过字符串字面量（单引号包裹）内的占位符。
    fn convert_placeholders(sql: &str, num_params: usize) -> String {
        if num_params == 0 {
            return sql.to_owned();
        }

        let mut result = String::with_capacity(sql.len());
        let mut param_index: usize = 1;
        let mut chars = sql.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                // 跳过字符串字面量
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
                // 处理 ? 占位符
                '?' if param_index <= num_params => {
                    use std::fmt::Write;
                    write!(result, ":{}", param_index).unwrap();
                    param_index += 1;
                }
                // 处理 $N 占位符（PostgreSQL 兼容）
                '$' if param_index <= num_params => {
                    // 跳过后续的数字
                    let mut num_str = String::new();
                    while let Some(&next) = chars.peek() {
                        if next.is_ascii_digit() {
                            num_str.push(next);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Ok(n) = num_str.parse::<usize>() {
                        use std::fmt::Write;
                        write!(result, ":{}", n).unwrap();
                        // 更新 param_index 追踪最大序号
                        if n > param_index {
                            param_index = n;
                        }
                        param_index += 1;
                    } else {
                        // 不是有效数字，原样保留
                        result.push('$');
                        result.push_str(&num_str);
                    }
                }
                other => {
                    result.push(other);
                }
            }
        }

        result
    }

    /// 核心执行方法：准备语句、执行并返回结果流。
    ///
    /// 区分查询和 DML：查询返回行流，DML 返回受影响行数。
    async fn execute_or_query(
        &mut self,
        sql: SqlStr,
        arguments: Option<&mut OracleArguments>,
    ) -> Result<
        impl futures_core::Stream<
            Item = Result<Either<OracleQueryResult, OracleRow>, Error>,
        > + use<'_>,
        Error,
    > {
        let mut logger = QueryLogger::new(sql.clone(), self.log_settings.clone());

        let num_params = arguments.as_ref().map(|a| a.len()).unwrap_or(0);
        let oracle_sql = Self::convert_placeholders(sql.as_str(), num_params);
        let is_query = Self::sql_is_query(sql.as_str());

        // 将 sqlx 统一参数转换为 sibyl 的 ToSql trait 对象
        let mut owned_args = arguments
            .map(build_sibyl_args)
            .unwrap_or_default();

        // 在 OCI 侧预编译语句
        let stmt = self.session.prepare(&oracle_sql).await.map_err(|e| {
            Error::from(OracleDbError::new(format!("prepare failed: {e}")))
        })?;

        if is_query {
            // 查询路径：先执行查询，再获取列元数据
            // （ATP 上 prepare 后 column_count() 不可用，必须在 query() 之后调用）
            let rows = run_query(&stmt, &mut owned_args).await?;

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

            // 将所有行读入内存（当前实现未使用流式游标）
            let mut collected: Vec<Either<OracleQueryResult, OracleRow>> = Vec::new();
            loop {
                match rows.next().await {
                    Ok(Some(sibyl_row)) => {
                        let mut values = Vec::with_capacity(num_cols);

                        for (i, col) in columns.iter().enumerate() {
                            let is_null = sibyl_row.is_null(i);
                            if is_null {
                                values.push(OracleValue {
                                    value: None,
                                    type_info: col.type_info.clone(),
                                });
                            } else {
                                let raw_bytes = match col.type_info {
                                    OracleTypeInfo::Raw | OracleTypeInfo::LongRaw | OracleTypeInfo::Blob => {
                                        let bytes: &[u8] = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get binary column {i}: {e}").into(),
                                            )
                                        })?;
                                        bytes.to_vec()
                                    }
                                    OracleTypeInfo::Date => {
                                        let d: sibyl::Date = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get date column {i}: {e}").into(),
                                            )
                                        })?;
                                        let text = d.to_string("YYYY-MM-DD HH24:MI:SS").map_err(|e| {
                                            Error::Decode(
                                                format!("failed to format date column {i}: {e}").into(),
                                            )
                                        })?;
                                        text.into_bytes()
                                    }
                                    OracleTypeInfo::Timestamp => {
                                        let ts: sibyl::Timestamp = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get timestamp column {i}: {e}").into(),
                                            )
                                        })?;
                                        let text = ts.to_string("YYYY-MM-DD HH24:MI:SS.FF", 6).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to format timestamp column {i}: {e}").into(),
                                            )
                                        })?;
                                        text.into_bytes()
                                    }
                                    OracleTypeInfo::TimestampTZ => {
                                        let ts: sibyl::TimestampTZ = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get timestamptz column {i}: {e}").into(),
                                            )
                                        })?;
                                        let text = ts.to_string("YYYY-MM-DD HH24:MI:SS.FF TZH:TZM", 6).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to format timestamptz column {i}: {e}").into(),
                                            )
                                        })?;
                                        text.into_bytes()
                                    }
                                    OracleTypeInfo::TimestampLTZ => {
                                        let ts: sibyl::TimestampLTZ = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get timestampltz column {i}: {e}").into(),
                                            )
                                        })?;
                                        let text = ts.to_string("YYYY-MM-DD HH24:MI:SS.FF TZH:TZM", 6).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to format timestampltz column {i}: {e}").into(),
                                            )
                                        })?;
                                        text.into_bytes()
                                    }
                                    _ => {
                                        let text: String = sibyl_row.get(i).map_err(|e| {
                                            Error::Decode(
                                                format!("failed to get column {i}: {e}").into(),
                                            )
                                        })?;
                                        text.into_bytes()
                                    }
                                };
                                values.push(OracleValue {
                                    value: Some(raw_bytes),
                                    type_info: col.type_info.clone(),
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
            // DML 路径：执行并返回影响行数
            let affected = run_execute(&stmt, &mut owned_args).await?;

            // 自动提交：不在显式事务中时，每个 DML 后自动 COMMIT
            if self.transaction_depth == 0 {
                run_execute_commit(&self.session).await?;
            }

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

    /// 关闭连接（sibyl 自动处理会话生命周期，此处为 no-op）。
    fn close(self) -> impl std::future::Future<Output = Result<(), Error>> + Send + 'static {
        async move { Ok(()) }
    }

    /// 强制关闭连接（no-op，同 `close`）。
    fn close_hard(self) -> impl std::future::Future<Output = Result<(), Error>> + Send + 'static {
        async move { Ok(()) }
    }

    /// 通过 `SELECT 1 FROM DUAL` 检查数据库是否存活。
    fn ping(&mut self) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        async move {
            let stmt = self.session.prepare("SELECT 1 FROM DUAL").await.map_err(|e| {
                Error::from(OracleDbError::new(format!("ping prepare failed: {e}")))
            })?;
            stmt.query(()).await.map_err(|e| {
                Error::from(OracleDbError::new(format!("ping query failed: {e}")))
            })?;
            Ok(())
        }
    }

    /// 开始事务（深度 0→1 时隐式开启，不做任何 SQL 操作）。
    fn begin(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    where
        Self: Sized,
    {
        Transaction::begin(self, None)
    }

    /// 使用自定义起始语句开始事务（透传给 `TransactionManager::begin`）。
    fn begin_with(
        &mut self,
        statement: impl sqlx_core::sql_str::SqlSafeStr,
    ) -> impl std::future::Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    where
        Self: Sized,
    {
        Transaction::begin(self, Some(statement.into_sql_str()))
    }

    fn shrink_buffers(&mut self) {}

    /// flush（no-op，Oracle 无类似 MySQL 的缓冲区刷新操作）。
    fn flush(&mut self) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        async move { Ok(()) }
    }

    /// 不需要 flush。
    fn should_flush(&self) -> bool {
        false
    }
}

impl<'c> Executor<'c> for &'c mut OracleConnection {
    type Database = Oracle;

    #[doc(hidden)]
    fn describe<'e>(
        self,
        sql: SqlStr,
    ) -> BoxFuture<'e, Result<sqlx_core::describe::Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let stmt = self.session.prepare(sql.as_str()).await.map_err(|e| {
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

            Ok(sqlx_core::describe::Describe {
                columns,
                parameters: None,
                nullable: Vec::new(),
            })
        })
    }

    /// 执行查询并返回混合流（DML 结果或行）。
    ///
    /// 从 `Execute` trait 对象提取 SQL 和参数，调用 `execute_or_query` 执行。
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
        let arguments = query.take_arguments();
        let sql = query.sql();

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

    /// 执行查询并返回可选的单行结果。
    ///
    /// 内部使用 `fetch_many` 流，只取第一个 `Either::Right(Row)`。
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
        let arguments = query.take_arguments();
        let sql = query.sql();

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
                if let Either::Right(row) = item
                    && result.is_none()
                {
                    result = Some(row);
                }
            }
            Ok(result)
        })
    }

    /// 预编译 SQL 并返回 `OracleStatement`（含列元数据）。
    ///
    /// 如果提供了参数类型信息，会先进行占位符转换。
    fn prepare_with<'e>(
        self,
        sql: SqlStr,
        _parameters: &'e [OracleTypeInfo],
    ) -> BoxFuture<'e, Result<OracleStatement, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            let num_params = _parameters.len();
            let oracle_sql = if num_params > 0 {
                OracleConnection::convert_placeholders(sql.as_str(), num_params)
            } else {
                sql.as_str().to_owned()
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
                sql: AssertSqlSafe(oracle_sql).into_sql_str(),
                columns,
            })
        })
    }
}

/// 将 sibyl 列类型映射到本 crate 的 `OracleTypeInfo`。
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

/// 将 sqlx 参数转换为 sibyl 的 `ToSql` trait 对象向量。
fn build_sibyl_args(args: &mut OracleArguments) -> Vec<Box<dyn sibyl::ToSql>> {
    args.buffer
        .values
        .iter()
        .map(|v| match v {
            OracleBindValue::Null => Box::new(None::<&str>) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Int(i) => Box::new(*i) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Float(f) => Box::new(*f) as Box<dyn sibyl::ToSql>,
            OracleBindValue::String(s) => Box::new(s.clone()) as Box<dyn sibyl::ToSql>,
            OracleBindValue::Bool(b) => Box::new(if *b { 1i32 } else { 0i32 }) as Box<dyn sibyl::ToSql>,
        })
        .collect()
}

#[allow(unsafe_code)]
#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // sql_is_query：判断 SQL 类型
    // -----------------------------------------------------------------------

    #[test]
    fn test_sql_is_query_select() {
        assert!(OracleConnection::sql_is_query("SELECT * FROM dual"));
        assert!(OracleConnection::sql_is_query("select 1 from dual"));
        assert!(OracleConnection::sql_is_query("  SELECT foo FROM bar"));
    }

    #[test]
    fn test_sql_is_query_with() {
        assert!(OracleConnection::sql_is_query("WITH cte AS (SELECT 1) SELECT * FROM cte"));
    }

    #[test]
    fn test_sql_is_query_not_query() {
        assert!(!OracleConnection::sql_is_query("INSERT INTO t VALUES (1)"));
        assert!(!OracleConnection::sql_is_query("UPDATE t SET x = 1"));
        assert!(!OracleConnection::sql_is_query("DELETE FROM t"));
        assert!(!OracleConnection::sql_is_query("MERGE INTO t USING ..."));
        assert!(!OracleConnection::sql_is_query(""));
    }

    // -----------------------------------------------------------------------
    // convert_placeholders：占位符转换（? → :n, $N → :n）
    // -----------------------------------------------------------------------

    #[test]
    fn test_convert_placeholders_no_args() {
        let sql = "SELECT 1 FROM dual";
        assert_eq!(OracleConnection::convert_placeholders(sql, 0), sql);
    }

    #[test]
    fn test_convert_placeholders_simple() {
        let sql = "SELECT * FROM t WHERE id = ? AND name = ?";
        let expected = "SELECT * FROM t WHERE id = :1 AND name = :2";
        assert_eq!(OracleConnection::convert_placeholders(sql, 2), expected);
    }

    #[test]
    fn test_convert_placeholders_skip_string_literals() {
        let sql = "SELECT ? as col FROM t WHERE name = '?'";
        let expected = "SELECT :1 as col FROM t WHERE name = '?'";
        assert_eq!(OracleConnection::convert_placeholders(sql, 1), expected);
    }

    #[test]
    fn test_convert_placeholders_multiple_literals() {
        let sql = "SELECT '?', ?, '?''?' FROM t WHERE x = ?";
        let expected = "SELECT '?', :1, '?''?' FROM t WHERE x = :2";
        assert_eq!(OracleConnection::convert_placeholders(sql, 2), expected);
    }

    #[test]
    fn test_convert_placeholders_none_provided() {
        let sql = "SELECT ? FROM t";
        // num_params 为 0 时不替换
        assert_eq!(OracleConnection::convert_placeholders(sql, 0), sql);
    }

    #[test]
    fn test_convert_placeholders_dollar_n() {
        let sql = "DELETE FROM t WHERE id < $1";
        let expected = "DELETE FROM t WHERE id < :1";
        assert_eq!(OracleConnection::convert_placeholders(sql, 1), expected);
    }

    #[test]
    fn test_convert_placeholders_dollar_n_multiple() {
        let sql = "SELECT $1, $2, $3 FROM t WHERE x = $4";
        let expected = "SELECT :1, :2, :3 FROM t WHERE x = :4";
        assert_eq!(OracleConnection::convert_placeholders(sql, 4), expected);
    }

    #[test]
    fn test_convert_placeholders_dollar_n_skip_string() {
        let sql = "SELECT '$1' FROM t WHERE id = $1";
        let expected = "SELECT '$1' FROM t WHERE id = :1";
        assert_eq!(OracleConnection::convert_placeholders(sql, 1), expected);
    }

    // -----------------------------------------------------------------------
    // build_sibyl_args：参数转换
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_sibyl_args_empty() {
        let mut args = OracleArguments::default();
        let result = build_sibyl_args(&mut args);
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_sibyl_args_types() {
        let mut args = OracleArguments::default();
        args.add(42i64).unwrap();
        args.add(3.14f64).unwrap();
        args.add(true).unwrap();
        args.add("hello").unwrap();

        let result = build_sibyl_args(&mut args);
        assert_eq!(result.len(), 4);
    }
}

/// 执行查询并返回行集。
///
/// # Safety
/// sibyl 的 `query()` 方法需要 `&mut [&mut dyn ToSql]` 参数，
/// 但 `Box<dyn ToSql>` 的生命周期无法直接表达。这里使用
/// `transmute` 将生命周期扩展到 `'static`，实际使用范围受
/// `stmt.query()` 调用的词法作用域限制，是安全的。
#[allow(unsafe_code)]
async fn run_query<'a>(
    stmt: &'a sibyl::Statement<'a>,
    owned_args: &mut Vec<Box<dyn sibyl::ToSql>>,
) -> Result<sibyl::Rows<'a>, Error> {
    let mut refs: Vec<&'static mut dyn sibyl::ToSql> = unsafe {
        owned_args.iter_mut()
            .map(|b| {
                let r: &mut dyn sibyl::ToSql = &mut **b;
                std::mem::transmute::<&mut dyn sibyl::ToSql, &'static mut dyn sibyl::ToSql>(r)
            })
            .collect()
    };
    stmt.query(&mut refs).await
        .map_err(|e| Error::from(OracleDbError::new(format!("query failed: {e}"))))
}

/// 执行 DML 并返回影响行数。
///
/// # Safety
/// 同 `run_query`，需要对生命周期进行 transmute 以适配 sibyl 签名。
#[allow(unsafe_code)]
async fn run_execute<'a>(
    stmt: &'a sibyl::Statement<'a>,
    owned_args: &mut Vec<Box<dyn sibyl::ToSql>>,
) -> Result<usize, Error> {
    let mut refs: Vec<&'static mut dyn sibyl::ToSql> = unsafe {
        owned_args.iter_mut()
            .map(|b| {
                let r: &mut dyn sibyl::ToSql = &mut **b;
                std::mem::transmute::<&mut dyn sibyl::ToSql, &'static mut dyn sibyl::ToSql>(r)
            })
            .collect()
    };
    stmt.execute(&mut refs).await
        .map_err(|e| Error::from(OracleDbError::new(format!("execute failed: {e}"))))
}

/// 执行 COMMIT（用于自动提交模式）。
#[allow(unsafe_code)]
async fn run_execute_commit(session: &sibyl::Session<'static>) -> Result<(), Error> {
    let commit_stmt = session.prepare("COMMIT").await.map_err(|e| {
        Error::from(OracleDbError::new(format!("prepare COMMIT failed: {e}")))
    })?;
    let mut empty_args: Vec<Box<dyn sibyl::ToSql>> = Vec::new();
    run_execute(&commit_stmt, &mut empty_args).await?;
    Ok(())
}
