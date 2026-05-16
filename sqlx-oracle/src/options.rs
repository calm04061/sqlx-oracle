use std::str::FromStr;
use std::fmt::Debug;
use std::time::Duration;

use futures_core::future::BoxFuture;
use log::LevelFilter;
use url::Url;

use sqlx_core::connection::{ConnectOptions, LogSettings};
use sqlx_core::error::Error;

use crate::OracleConnection;

/// Oracle 连接配置。
///
/// 保存数据库 URL 和日志设置，通过 `ConnectOptions` trait
/// 供 sqlx 框架统一调用。
#[derive(Debug, Clone)]
pub struct OracleConnectOptions {
    pub(crate) database_url: String,
    pub(crate) log_settings: LogSettings,
}

impl OracleConnectOptions {
    pub fn new() -> Self {
        Self {
            database_url: String::new(),
            log_settings: LogSettings::default(),
        }
    }
}

impl Default for OracleConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for OracleConnectOptions {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Error> {
        Ok(Self {
            database_url: url.to_owned(),
            log_settings: LogSettings::default(),
        })
    }
}

impl ConnectOptions for OracleConnectOptions {
    type Connection = OracleConnection;

    /// 从已解析的 URL 创建配置。
    fn from_url(url: &Url) -> Result<Self, Error> {
        Ok(Self {
            database_url: url.to_string(),
            log_settings: LogSettings::default(),
        })
    }

    /// 建立连接。
    fn connect(&self) -> BoxFuture<'_, Result<OracleConnection, Error>> {
        Box::pin(async move {
            OracleConnection::establish(&self.database_url, self.log_settings.clone()).await
        })
    }

    fn log_statements(mut self, level: LevelFilter) -> Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(mut self, level: LevelFilter, duration: Duration) -> Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
