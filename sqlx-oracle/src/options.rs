use std::path::PathBuf;
use std::str::FromStr;
use std::fmt::Debug;
use std::time::Duration;

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
    fn connect(&self) -> impl std::future::Future<Output = Result<OracleConnection, Error>> + Send + '_ {
        async move {
            OracleConnection::establish(
                &self.database_url,
                self.log_settings.clone(),
                self.wallet_path.clone(),
            ).await
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let opts = OracleConnectOptions::new();
        assert!(opts.database_url.is_empty());
        assert!(opts.wallet_path.is_none());
    }

    #[test]
    fn test_from_str_basic() {
        let url = "oracle://user:pass@localhost:1521/XEPDB1";
        let opts = OracleConnectOptions::from_str(url).unwrap();
        assert_eq!(opts.database_url, url);
        assert!(opts.wallet_path.is_none());
    }

    #[test]
    fn test_from_str_with_wallet() {
        let url = "oracle://admin:pass@myatp_high?wallet=/path/to/wallet";
        let opts = OracleConnectOptions::from_str(url).unwrap();
        assert_eq!(opts.database_url, url);
        assert_eq!(opts.wallet_path, Some(PathBuf::from("/path/to/wallet")));
    }

    #[test]
    fn test_from_url_basic() {
        let url = Url::parse("oracle://user:pass@localhost:1521/FREEPDB1").unwrap();
        let opts = OracleConnectOptions::from_url(&url).unwrap();
        assert!(opts.wallet_path.is_none());
    }

    #[test]
    fn test_from_url_with_wallet() {
        let url = Url::parse("oracle://admin:pass@myatp_high?wallet=/path/to/wallet").unwrap();
        let opts = OracleConnectOptions::from_url(&url).unwrap();
        assert_eq!(opts.wallet_path, Some(PathBuf::from("/path/to/wallet")));
    }

    #[test]
    fn test_from_url_no_wallet_param() {
        let url = Url::parse("oracle://user:pass@host:1521/service").unwrap();
        let opts = OracleConnectOptions::from_url(&url).unwrap();
        assert!(opts.wallet_path.is_none());
    }

    #[test]
    fn test_clone() {
        let a = OracleConnectOptions {
            database_url: "oracle://u:p@h/s".to_string(),
            log_settings: LogSettings::default(),
            wallet_path: Some(PathBuf::from("/wallet")),
        };
        let b = a.clone();
        assert_eq!(a.database_url, b.database_url);
        assert_eq!(a.wallet_path, b.wallet_path);
    }

    #[test]
    fn test_debug() {
        let opts = OracleConnectOptions {
            database_url: "oracle://u:p@h/s".to_string(),
            log_settings: LogSettings::default(),
            wallet_path: None,
        };
        let debug = format!("{opts:?}");
        assert!(debug.contains("oracle://"));
    }

    #[test]
    fn test_log_statements() {
        let opts = OracleConnectOptions::new()
            .log_statements(LevelFilter::Debug);
        assert_eq!(opts.log_settings.statements_level, LevelFilter::Debug);
    }

    #[test]
    fn test_log_slow_statements() {
        let opts = OracleConnectOptions::new()
            .log_slow_statements(LevelFilter::Warn, Duration::from_secs(5));
        assert_eq!(opts.log_settings.slow_statements_level, LevelFilter::Warn);
        assert_eq!(opts.log_settings.slow_statements_duration, Duration::from_secs(5));
    }
}
