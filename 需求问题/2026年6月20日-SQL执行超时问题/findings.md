# 发现与决策

## 需求
补齐剩余数据库驱动的服务端 cancel 机制，确保用户点击「停止」按钮后服务端查询真正终止。

## 研究发现

### PostgreSQL 取消分析
- 当前 `execute_query_with_max_rows(&Pool, sql, max_rows)` 取 `&Pool`，内部 `pool.get().await` 拿 Client
- Client 由 deadpool 管理，外部无法直接 close/断开 TCP
- 取消时 `remove_pool_by_key` 仅丢弃池，**服务端查询仍在运行**
- 需要新函数 `execute_query_with_owned_client(pool, sql, max_rows)` 返回 `(Result, Client)`
- 或使用 `deadpool::Object` 的 `.take()` / `Pool::try_get()` — 但 deadpool 没有暴露底层连接所有权
- 最可行方案：`pool.get().await` 拿到 `deadpool::Object<Client>`，查询完或取消时 `drop(object)` 或 `object.take().close()`
- 实际 `deadpool_postgres::Object` 实现了 `Deref<Target = Client>`，但 `Client::close()` 是 `&mut self`
- 可通过 `deadpool::Object::take()` 获取底层 `Client` 的所有权（`deadpool 0.12+`），然后 `client.close().await`

### SQL Server 取消分析
- `PoolKind::SqlServer(Arc<tokio::sync::Mutex<SqlServerClient>>)`
- `SqlServerClient` 是 `tiberius::Client<Compat<TcpStream>>` 的别名
- `tiberius::Client` 无自定义 Drop → TcpStream 被丢时发送 FIN → SQL Server 检测到 TCP 断开 → 杀掉查询
- `Arc<Mutex<>>` 不阻挡 TCP close：pool discard → last Arc drop → SqlServerClient drop → TcpStream FIN → 查询终止
- **需修复的 bug：** pre-lock cancel（第 941 行）提前 return 跳过 `remove_pool_by_key`，Arc 保留在 pool 中，TCP 不断开。补上 `remove_pool_by_key` 即可。
- 结论：**TCP close cancel 已可实现**，无需重构为 Pool 模式

### ExternalDriver 取消分析
- 通过 PluginDriverSession (JSON-RPC over stdin/stdout) 调用外部插件进程
- `session.invoke_with_timeout()` 内部在超时时 `self.kill().await` 杀子进程，但 `wait_for_query_opt` cancel 时只 drop future，不杀进程
- `PluginDriverSession` 无 `Drop` impl → pool discard 也不杀进程
- **修复：**
  1. 添加 `Drop for PluginDriverSession` → `process.child.start_kill()` 同步杀子进程（与 AgentDriverClient 模式一致）
  2. ExternalDriver dispatch 加 `is_canceled` 检查 + `remove_pool_by_key`（之前完全缺失）
- 结论：关闭 stdin/stdout 管道不足以终止插件进程，必须显式 `start_kill()`。添加 Drop 后 cancel 已可用。

### Agent/JDBC 取消
- ✅ 已确认 `remove_pool_by_key` → Arc drop → `kill()` 杀 JVM 进程 → JDBC 查询终止
- 无需 agent 侧改动

### HTTP 类驱动（ClickHouse, ES, VectorDb, InfluxDb, Rqlite, Turso）
- ✅ Cancel 时 future drop → reqwest 断开 TCP → 服务端自动终止查询
- 无需额外处理

## 技术决策
| 决策 | 理由 |
|------|------|
| PostgreSQL: 新函数 `execute_query_on_client` / `execute_query_with_schema_on_client` 取 `&Client`；`take_postgres_client` 用 `deadpool::Object::take()` 断开 TCP | deadpool 0.12+ 支持 `.take()` 获取底层连接所有权 |
| PostgreSQL: dispatch 改为 `pool.get()→client` → `wait_for_query_opt` 包装查询 → cancel 时 `take_postgres_client(client)` | 需要 `deadpool::managed::Object::take(client)` 提取 `tokio_postgres::Client` |
| SQL Server: pre-lock cancel 补 `remove_pool_by_key` → Arc 归零 → tiberius Client drop → TcpStream FIN → 查询终止 | `Arc<Mutex<>>` 不阻挡 TCP close；仅 pre-lock cancel 路径有遗漏 |
| ExternalDriver: 添加 `Drop for PluginDriverSession` → `start_kill()`；dispatch 补 pool discard | 关闭管道不足以终止插件进程，需显式 start_kill |

## 遇到的问题
| 问题 | 解决方案 |
|------|---------|
| deadpool_postgres::Object 不直接暴露 close() | 使用 `.take()` 获取原生 Client（需 deadpool 0.12+） |

## 资源
- `crates/dbx-core/src/query.rs` — `do_execute()` dispatch
- `crates/dbx-core/src/db/postgres.rs` — `execute_query_with_max_rows()`
- `crates/dbx-core/src/db/sqlserver.rs` — `SqlServerClient` 定义
- deadpool 文档: https://docs.rs/deadpool-postgres/
