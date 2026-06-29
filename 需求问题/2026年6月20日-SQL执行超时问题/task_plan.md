# 任务计划：SQL执行超时与连接状态不同步修复

## 目标
移除后端的 30s 自动超时中断逻辑（查询应持续运行直到完成或用户手动中断），为各驱动补齐用户触发的查询中断能力，并修复前端连接状态与后端池不同步的问题。

## 当前阶段
阶段 10

## 各阶段

### 阶段 1-6：（已完成）
- [x] 移除 30s 自动超时
- [x] MySQL/Agent TCP 断开取消
- [x] PostgreSQL/SQL Server pool discard 取消
- [x] HTTP 类驱动 future drop 取消
- [x] 前端连接状态同步
- [x] 编译验证
- **状态：** complete

### 阶段 7：PostgreSQL 服务端 cancel（owned client）
- [x] 7.1 分析 deadpool 版本（0.12.3）支持 `.take()` 获取底层 `tokio_postgres::Client`
- [x] 7.2 新增 `execute_query_on_client` / `execute_query_with_schema_on_client`（取 `&Client` 执行查询）
- [x] 7.3 新增 `take_postgres_client`（`deadpool_postgres::Object::take()` + drop）
- [x] 7.4 PostgreSQL dispatch 改为：`pool.get()` 先拿 client → `wait_for_query_opt` 包装查询 → cancel 时 `take_postgres_client(client)` 断开 TCP
- [x] 7.5 添加 `deadpool = "0.12"` 为直接依赖（已锁文件中）
- [x] 7.6 cargo check 编译通过（仅 1 个 pre-existing warning）
- **状态：** complete

### 阶段 8：SQL Server 服务端 cancel
- [x] 8.1 分析 `SqlServerClient` 和 `Arc<Mutex<>>` 架构
- [x] 8.2 发现 pre-lock cancel 的提前 return 未调用 `remove_pool_by_key`，Arc 不释放，TCP 不断开
- [x] 8.3 修复：在 cancel 路径上加 `state.remove_pool_by_key(pool_key).await` 使其与其他 cancel 路径一致
- **关键发现：** TCP close cancel 可行。`tiberius::Client` 无自定义 Drop → `TcpStream` 被丢时发送 FIN → SQL Server 杀掉查询。pre-lock cancel 路径的 `remove_pool_by_key` 遗漏是唯一 bug。
- **状态：** complete

### 阶段 9：ExternalDriver 取消
- [x] 9.1 分析插件协议（JSON-RPC over stdin/stdout）：原生不支持 cancel RPC
- [x] 9.2 发现 `PluginDriverSession` 无 `Drop` impl → pool discard 不杀 child process
- [x] 9.3 实现：
  - `plugins.rs`: 添加 `Drop for PluginDriverSession` → `process.child.start_kill()` 杀子进程
  - `query.rs`: ExternalDriver dispatch 加 `is_canceled` 检查 + `remove_pool_by_key`（与 Agent 模式一致）
- **关键发现：** 关闭 stdin/stdout 管道不足以终止插件进程。必须显式 `start_kill()`。添加 Drop 后，pool discard → Arc 归零 → Drop kill 子进程 → 查询取消。
- **状态：** complete

### 阶段 10：验证
- [x] 10.1 cargo check 编译验证（仅 1 个 pre-existing `ssh_agent_sock_path` warning）
- [ ] 10.2 手动验证各驱动 cancel 行为
- **状态：** in_progress

## 关键问题
1. ~~为什么存储过程 30s 被中断？~~ → 后端 `QUERY_TIMEOUT` 在 `wait_for_query_opt` 中 drop future ✅
2. ~~为什么关闭连接后重新打开仍超时？~~ → 前端 `connectedIds` 未清理，`ensureConnected` 直接 return 不重建池 ✅
3. PostgreSQL deadpool 版本是否支持 `Object::take()`？需要检查 Cargo.lock
4. ~~SQL Server 能否从 `Arc<Mutex<>>` 模式重构为池模式？工作量评估~~ → TCP close cancel 通过 pool discard 即可实现，无需重构 ✅
5. ~~ExternalDriver 插件协议是否已有 cancel 设计？~~ → 无原生支持。通过 Drop impl + pool discard 实现进程杀死取消 ✅

## 已做决策
| 决策 | 理由 |
|------|------|
| 移除 30s 自动超时中断 | 用户需求：只有用户点击中断才中断 |
| MySQL: TCP disconnect 而非 KILL QUERY | 更可靠，无需额外连接，无权限要求 |
| PostgreSQL: 新函数返回 owned Client | deadpool::Object::take() 可获取底层连接所有权 |
| SQL Server: pool discard → Arc 归零 → tiberius Client drop → TcpStream FIN → SQL Server 杀查询 | pre-lock cancel 路径需补 `remove_pool_by_key`；`Arc<Mutex<>>` 不阻挡 TCP close |
| Agent/JDBC: pool discard → Arc drop → kill() | 杀 JVM 进程后 JDBC 查询自然终止 |
| ExternalDriver: 添加 Drop impl → pool discard 杀子进程 | 关闭管道不足以终止插件进程，需要显式 `start_kill()` |

## 遇到的错误
| 错误 | 尝试次数 | 解决方案 |
|------|---------|---------|
| — | 0 | — |

## 备注
- PostgreSQL 是否支持 `.take()` 直接影响方案可行性
- SQL Server 重构为池模式直接影响 dbx-core 架构，需谨慎评估
