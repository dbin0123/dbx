# 发现与决策

## 需求
修复所有数据库驱动（MySQL, PostgreSQL, SQL Server, Redis, MongoDB, ClickHouse, Elasticsearch, InfluxDB, VectorDB, Agent/Oracle）的连接稳定性问题。核心症状：连接断开后无法自动恢复，一个连接失效导致全部失效，需要重启软件。

## 研究发现

### 1. keepalive 默认关闭（全局通病）
- 位置：`crates/dbx-core/src/models/connection.rs:208-210`
- `default_keepalive_interval_secs()` 返回 `0`，所有驱动不启动保活
- 位置：`crates/dbx-core/src/connection.rs:337-340`
- `start_keepalive_task` 在 `interval_secs == 0` 时直接 return

### 2. refresh_connections 仅覆盖 MySQL 和 PostgreSQL
- 位置：`crates/dbx-core/src/connection.rs:1175-1178`
- `matches!(pool, PoolKind::Mysql(..) | PoolKind::Postgres(..))` 过滤
- 前端 `useVisibilityChange.ts:14` 调用 refresh 但只对两种驱动有效

### 3. 健康检查无超时（连锁失效的根因）
- MySQL `db/mysql.rs:1589-1600`：`conn.ping().await` 无超时
- PostgreSQL `connection.rs:1192-1204`：`p.get().await` + `simple_query` 无超时
- SQL Server `db/sqlserver.rs:582-586`：`simple_query` 无超时
- MongoDB `db/mongo_driver.rs:62-69`：`_timeout` 参数被忽略
- 对比：Redis/ClickHouse/Elasticsearch 已有超时包装

### 4. 元数据操作重连覆盖不全
- `schema.rs` 的 `retry_metadata_connection` 只包装了 6 个操作
- 缺失重连：`get_columns`, `list_indexes`, `list_foreign_keys`, `list_triggers`, `list_functions`, `list_sequences`, `list_rules`, `list_owners`, `get_table_ddl`, `get_object_source`

### 5. close_pool_kind 形同虚设
- 位置：`connection.rs:1455-1480`
- 只有 MySQL/Postgres/Agent/ExternalDriver 有实际操作
- SQL Server, Redis, MongoDB, ClickHouse, Elasticsearch, VectorDB, InfluxDB, SQLite, Rqlite, Turso 全部是空 `{}`

### 6. remove_stale_connection_pool 覆盖不全
- 位置：`connection.rs:890-955`
- 健康检查仅覆盖 MySQL, SQL Server, Redis, MongoDB
- PostgreSQL, ClickHouse, Elasticsearch, InfluxDB, VectorDB, Rqlite, Turso 全部返回 false

### 7. 查询重连逻辑不一致
- `query.rs:1175-1186` 的 `ReconnectAndRetry` 在外层触发
- 但 SQL Server/Agent 内部仅 remove_pool 不重试
- ClickHouse/Elasticsearch/InfluxDB/VectorDB 既不重试也不移除池

### 8. Windows 系统事件处理不足
- `src-tauri/src/lib.rs:695-703`：仅 macOS 触发 refresh
- `useVisibilityChange.ts:13-22`：`document.hidden` 在 Windows 休眠/唤醒可能不触发

## 技术决策
| 决策 | 理由 |
|------|------|
| 按 P0 → P1 → P2 顺序修复 | P0 解决连锁失效根因，P1 解决恢复问题，P2 优化 |
| keepalive 默认值改为 60s | 多数服务端超时在 120-600s，60s 安全且不过度 |
| 健康检查超时统一使用 5s | 本地/局域网 ping 应在 1s 内完成，5s 足够余量 |
| 使用 `tokio::time::timeout` 包装无超时的操作 | 项目已使用 tokio，保持一致性 |
| close_pool_kind 对 HTTP 客户端使用 drop | ClickHouse/Elasticsearch 客户端 drop 会自动关闭 |

## 资源
- 项目根目录：`D:\Developments\jetbrains\workspace\rust\dbx`
- 核心模块：`crates/dbx-core/src/`
- 数据库驱动：`crates/dbx-core/src/db/`

---
*每执行2次查看/浏览器/搜索操作后更新此文件*
