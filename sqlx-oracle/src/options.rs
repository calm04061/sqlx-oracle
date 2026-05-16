use std::path::PathBuf;
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
///
/// # URL 格式
///
/// ## 直连 (ezconnect)
/// ```text
/// oracle://user:password@host:port/service_name
/// ```
///
/// ## 通过 TNS 别名连接（需要 tnsnames.ora）
/// ```text
/// oracle://user:password@tns_alias
/// ```
///
/// ## ATP/TCPS 连接（需要 Oracle wallet）
/// ```text
/// oracle://user:password@tns_alias?wallet=/path/to/wallet
/// ```
/// `wallet` 参数指向包含 `tnsnames.ora`、`sqlnet.ora`、`ewallet.p12` 的目录。
#[derive(Debug, Clone)]
pub struct OracleConnectOptions {
    pub(crate) database_url: String,
    pub(crate) log_settings: LogSettings,
    /// Oracle wallet 目录路径（用于 ATP/TCPS SSL 连接）
    pub(crate) wallet_path: Option<PathBuf>,
}

impl OracleConnectOptions {
    pub fn new() -> Self {
        Self {
            database_url: String::new(),
            log_settings: LogSettings::default(),
            wallet_path: None,
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
        let wallet_path = Url::parse(url).ok()
            .and_then(|u| u.query_pairs()
                .find(|(key, _)| key == "wallet")
                .map(|(_, val)| PathBuf::from(val.into_owned()))
            );
        Ok(Self {
            database_url: url.to_owned(),
            log_settings: LogSettings::default(),
            wallet_path,
        })
    }
}

impl ConnectOptions for OracleConnectOptions {
    type Connection = OracleConnection;

    /// 从已解析的 URL 创建配置。
    ///
    /// 从查询参数中提取：
    /// - `wallet` — Oracle wallet 目录路径
    fn from_url(url: &Url) -> Result<Self, Error> {
        let wallet_path = url.query_pairs()
            .find(|(key, _)| key == "wallet")
            .map(|(_, val)| PathBuf::from(val.into_owned()));
        Ok(Self {
            database_url: url.to_string(),
            log_settings: LogSettings::default(),
            wallet_path,
        })
    }

    /// 建立连接。
    fn connect(&self) -> BoxFuture<'_, Result<OracleConnection, Error>> {
        Box::pin(async move {
            OracleConnection::establish(
                &self.database_url,
                self.log_settings.clone(),
                self.wallet_path.clone(),
            ).await
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
