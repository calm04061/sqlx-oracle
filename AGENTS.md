# sqlx-oracle — agent guide

## Workspace

Two crates: `sqlx-oracle` (driver lib) + `sqlx-oracle-unit` (integration tests).
Edition 2024. Resolver 3.

## Build

`build.rs` auto-downloads Oracle Instant Client Basic Light for the target platform
and links `libclntsh`. No manual OCI install needed. Works on macOS (arm64/x86_64 via DMG),
Linux (x86_64/aarch64 via ZIP), Windows (x86_64).

Build may take a while first time (download + unzip + codesign on macOS).

```sh
cargo build
```

## Tests

Unit tests live inline in source files (`#[cfg(test)] mod tests`).

```sh
cargo test -p sqlx-oracle --lib      # unit tests only, no DB needed
```

Integration tests live in `sqlx-oracle-unit/tests/integration.rs` and require a live Oracle DB.
They must run single-threaded (shared temp tables):

```sh
cargo test -p sqlx-oracle-unit --test integration -- --test-threads=1 --nocapture
```

`DATABASE_URL` env var (or `.env`) points to the Oracle instance. Example format:
`oracle://user:password@host:port/service_name`.

A working `.env` is present in the repo (ATP wallet in `wallet/`).

## Architecture

- `sqlx_oracle::Oracle` implements `sqlx::Database` — zero-size type marker.
- `OracleConnection` wraps `sibyl::Session`. OCI environment is a global `OnceCell`.
- Placeholder conversion: `?` and `$N` → Oracle `:n` format (see `connection.rs:convert_placeholders`).
- `build_sibyl_args()` converts sqlx bind values to sibyl `ToSql` trait objects.
- Connection URL supports TNS aliases and `?wallet=` for ATP/TCPS.
- NLS session params set on connect for ISO 8601 date compatibility.

## Transactions

Oracle uses **implicit transactions**: the first DML after connect automatically starts
a transaction. `begin()` on depth 0 emits no SQL — the implicit tx is already active.
Nesting via `conn.begin()` creates `SAVEPOINT _sqlx_savepoint_{depth}`.

| Depth | begin               | commit                | rollback                     |
|-------|----------------------|-----------------------|------------------------------|
| 0→1   | *(implicit, no SQL)* | `COMMIT`              | `ROLLBACK`                   |
| 1→2   | `SAVEPOINT _sqlx_savepoint_1` | `RELEASE SAVEPOINT _sqlx_savepoint_1` | `ROLLBACK TO SAVEPOINT _sqlx_savepoint_1` |
| n→n+1 | `SAVEPOINT _sqlx_savepoint_{n}` | `RELEASE SAVEPOINT _sqlx_savepoint_{n}` | `ROLLBACK TO SAVEPOINT _sqlx_savepoint_{n}` |

Implementation at `transaction.rs:OracleTransactionManager`. Depth tracked in
`OracleConnection.transaction_depth`.

Integration tests cover rollback via `transaction_rollback` — shared temp tables
require `--test-threads=1`.

Note: `begin_with()` on the `Connection` trait allows a custom start statement
(rarely used — passes through to `OracleTransactionManager::begin`).

## Important quirks

- `#![deny(unsafe_code)]` at crate root; `connection.rs` has `#[allow(unsafe_code)]` for sibyl transmute — don't add new unsafe elsewhere.
- Oracle treats empty string as NULL — reflected in integration test `string_roundtrip`.
- `format_placeholder` outputs `$N` style (sqlx convention), then `convert_placeholders` rewrites `$N`/`?` to `:N` for Oracle.
- `close()` and `close_hard()` are no-ops (sibyl handles session lifecycle).
- `flush()` and `shrink_buffers()` are no-ops.
- Column lookup is case-insensitive (via `eq_ignore_ascii_case`).

## CI

Only triggers on `/oc` or `/opencode` in issues/comments/PR reviews. Uses opencode v1.14.31.
