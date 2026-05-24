# sqlx-oracle

为 [sqlx](https://github.com/launchbadge/sqlx) 异步数据库框架提供的 Oracle 数据库驱动，基于 [sibyl](https://crates.io/crates/sibyl)（Oracle OCI 非阻塞绑定），纯 Rust + tokio 实现。

## 特性

- 完全实现 `sqlx::Database` trait，可与 sqlx 查询构建器无缝配合
- 支持四种连接方式：ezconnect、TNS 别名、ATP/TCPS 钱包
- `build.rs` 自动下载 Oracle Instant Client Basic Light，无需手动安装 OCI
- 支持嵌套事务（SAVEPOINT）
- 类型编解码：字符串、整数（i8-u64）、浮点、布尔、二进制、chrono 时间类型
- 列名大小写不敏感查找
- 默认 `#![deny(unsafe_code)]`（仅 sibyl transmute 例外）

## 连接 URL 格式

### 直连 (ezconnect)

```
oracle://user:password@host:port/service_name
```

### TNS 别名（需要 tnsnames.ora）

```
oracle://user:password@tns_alias
```

### ATP/TCPS 连接（需要 Oracle wallet）

```
oracle://user:password@tns_alias?wallet=/path/to/wallet_dir
```

`wallet` 参数指向包含 `tnsnames.ora`、`sqlnet.ora`、`ewallet.p12` 的目录。驱动自动设置 `TNS_ADMIN` 环境变量。

## 使用示例

```rust
use sqlx::Connection;
use sqlx_oracle::OracleConnection;

let mut conn = OracleConnection::connect(
    "oracle://scott:tiger@localhost:1521/FREEPDB1"
).await.unwrap();

let row = sqlx::query("SELECT 1 AS val FROM DUAL")
    .fetch_one(&mut conn)
    .await.unwrap();

let val: i64 = row.get("val");
println!("{val}");
```

## 构建

构建脚本自动下载 Oracle Instant Client Basic Light 并链接 `libclntsh`。无需手动安装 OCI。

```sh
cargo build
```

支持平台：
- macOS arm64 / x86_64（DMG）
- Linux x86_64 / aarch64（ZIP）
- Windows x86_64（ZIP）

## 测试

### 单元测试（无需数据库）

```sh
cargo test -p sqlx-oracle --lib
```

### 集成测试（需要真实 Oracle 数据库）

```sh
cargo test -p sqlx-oracle-unit --test integration -- --test-threads=1 --nocapture
```

集成测试需要 `DATABASE_URL` 环境变量（或 `.env` 文件），例如：
```
DATABASE_URL=oracle://user:password@host:1521/service_name
```

> 集成测试共享临时表，必须单线程运行。

## 架构

| 模块 | 职责 |
|---|---|
| `database.rs` | 零尺寸 `Oracle` 类型，实现 `Database` trait |
| `connection.rs` | 连接管理、占位符转换（`?`/`$N` → `:N`）、查询/DML 执行 |
| `transaction.rs` | 隐式事务 + SAVEPOINT 嵌套事务管理 |
| `arguments.rs` | 绑定参数缓冲区（Null/String/Int/Float/Bool） |
| `encode_decode.rs` | Rust 类型与 Oracle 值之间的编解码 |
| `type_info.rs` | Oracle 数据类型枚举与兼容性检查 |
| `value.rs` | 列值（`OracleValue` / `OracleValueRef`） |
| `row.rs` | 行数据（按序号/名称访问） |
| `column.rs` | 列元数据 |
| `statement.rs` | 预编译语句 |
| `query_result.rs` | DML 影响行数 |
| `options.rs` | 连接配置（`OracleConnectOptions`） |
| `error.rs` | 数据库错误包装 |
| `build.rs` | 自动下载并链接 Oracle Instant Client |

### 事务

Oracle 使用隐式事务：第一个 DML 自动开启。`begin()` 在深度 0 时不发 SQL。嵌套事务使用 `SAVEPOINT`。

| 深度 | begin | commit | rollback |
|---|---|---|---|
| 0→1 | 隐式（无 SQL） | `COMMIT` | `ROLLBACK` |
| 1→2 | `SAVEPOINT _sqlx_savepoint_1` | `RELEASE SAVEPOINT _sqlx_savepoint_1` | `ROLLBACK TO SAVEPOINT _sqlx_savepoint_1` |
| n→n+1 | `SAVEPOINT _sqlx_savepoint_{n}` | `RELEASE SAVEPOINT _sqlx_savepoint_{n}` | `ROLLBACK TO SAVEPOINT _sqlx_savepoint_{n}` |

### 占位符转换

sqlx 使用 `?` 和 `$N` 占位符，Oracle 使用 `:N` 命名参数。`connection.rs:convert_placeholders` 自动转换，并跳过字符串字面量内的占位符。

### 类型映射

| Rust 类型 | Oracle 类型 | 备注 |
|---|---|---|
| `String` / `&str` | `VARCHAR2` | |
| `i8`..`u64` | `NUMBER` | |
| `f64` | `NUMBER` | |
| `bool` | `NUMBER` (0/1) | |
| `Vec<u8>` | `RAW` | 参数编码为 hex 字符串；RAW 列直接读取字节 |
| `NaiveDateTime` | `TIMESTAMP` | NLS 设为 ISO 8601 格式 |
| `NaiveDate` | `DATE` | |
| `NaiveTime` | `DATE` | 固定 2000-01-01 日期前缀 |
| `DateTime<Utc>` | `TIMESTAMP WITH TIME ZONE` | |

## 许可

MIT
