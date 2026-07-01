# 发现与决策

### 阶段 5 实施总结

**新增类型与结构 (data_compare.rs):**
- `DegradationLevel` (Full/Sample/SkipWithRisk) — 自动降级三级策略
- `DegradationThreshold` — 降级阈值配置（默认 100K/10M/10K/500）
- `SamplingStrategy` (Random/ExtremeValues/Hybrid) — 方言感知采样策略
- `VerifyDataOptions` — 带降级/采样控制的验证选项
- `VerifyDataResult` — 完整验证结果（含置信度/校验和/方法）

**新增类型与结构 (correction.rs):**
- `CorrectionStep` / `CorrectionStepType` — 单个订正步骤（Schema/Data/Checkpoint）
- `CorrectionRiskLevel` (Safe/Caution/Dangerous/Blocked) — 订正风险等级
- `CorrectionStrategy` (StructureFirst/DataFirst/Interleaved) — 联合编排水位策略
- `JointCorrectionPlan` / `JointCorrectionOptions` — 统一订正计划

**关键设计决策:**
| 决策 | 理由 |
|------|------|
| 降级阈值通过可选字段接入 `DataCompareFromTablesOptions` | 不破坏现有 API，向后兼容 |
| 采样 SQL 按方言差异化生成 | 利用各数据库原生采样能力 |
| 校验和通过 Rust SHA256 计算 | 跨方言一致性，避免方言 checksum 函数差异 |
| correction 模块独立于 data_compare | 关注点分离，可扩展为独立订正引擎 |

**已修复的问题:**
- `target_count_rows` → `target_row_count` 变量名修正
- `joint_correction_plan_serialization` 断言适配 camelCase
- `DatabaseType` match 覆盖核心 SQL 方言采样

### 阶段 4 实施总结

**新增类型与结构 (schema_diff.rs):**
- `DependencyGraph` / `DependencyNode` — 从 ForeignKeyInfo 构建的有向无环图，支持拓扑排序和覆盖率评分
- `DiffNode` — 包装 TableDiff + DiffDirection + dependency_order + rename 元数据
- `RollbackGraph` — 前向/回滚变更的双向图，含一致性验证
- `BatchPattern` — 通配符/正则批量命名模式
- `PermissionInfo` / `PermissionDiff` — 权限 DDL 感知
- `ColumnCompatibilityWarning` / `ColumnConversionRisk` — 方言感知类型兼容评分
- `ShardStrategy` / `ResourceConstraint` / `AdaptiveScheduler` — 分片并行与资源调度

**关键设计决策:**
| 决策 | 理由 |
|------|------|
| 所有新功能通过可选字段 + builder 模式集成 | 不破坏现有 Tauri/dbx-web API |
| 拓扑排序使用 Kahn 算法 + fallback 环形处理 | 外键可能形成循环，需降级不丢失表 |
| rename 检测使用贪心匹配（高分优先，已匹配排除） | 避免一对多虚假匹配 |
| 回滚图 invert_diff 直接翻转 diff_type 并交换 source/target | 简洁，一致性通过 validate_consistency 验证 |
| rayon 分片并行仅在 shard_count > 1 时启用 | 避免并行开销 |
| diff_columns_with_compatibility 仅在 dialect 提供时生效 | 维持原有 binary 比对默认行为 |

**已修复的问题:**
- 拓扑排序 in_degree 方向修正 (node depends_on → node in_degree++)
- detect_renames source/target detail 映射方向修正
- 上游 3 处 struct literal 添加 `..Default::default()` 适配新字段

## 需求

基于 V4 详细设计文档（脚本比对设计方案V4.md），实现企业级数据库结构比对与同步工具，核心涵盖：
- 多方言适配（MySQL、PostgreSQL、SQLite 等）
- 五层配置继承（Global → Team → Project → Env → Task）
- SQL 解析与 AST 安全透传
- 双向 Diff 与回滚图生成
- 数据验证与联合订正
- 状态校准与智能基线重置
- 在线安全评估与危险 DDL 兜底
- 状态持久化与加密签名
- 脚本生成（幂等、回滚、断点续传）

## 研究发现

### 方言适配策略
- **sqlparser-rs**：Rust 原生 SQL 解析器，支持 MySQL、PostgreSQL、SQLite、ANSI 等多种方言，可扩展自定义方言
- **方言能力描述符**：需要定义 DialectCapabilityDescriptor 结构体，描述 DDL/DML 支持度、类型映射、索引/约束语法差异
- **映射推导引擎**：基于方言能力矩阵自动推导跨方言类型映射

### 配置继承体系
- 5 层结构：Global > Team > Project > Env > Task
- 表达式引擎需支持：`${env.VAR}`、`${ref:path}`、`${eval:expr}` 三种模式
- 严格标签防穿透：每个业务标签必须显式声明可穿透层级，否则默认阻断

### 差异计算核心
- 依赖图感知：外键、视图依赖、触发器依赖需构建 DAG 进行拓扑排序
- 重命名检测：基于 Levenshtein 距离 + 列数/类型相似度的启发式匹配
- 双向 Diff：前向变更图 + 回滚变更图，确保可逆性
- 分片并行：按 Schema/表级别分片，分布式执行比对任务

### 安全评估
- 4 个危险等级：SAFE / CAUTION / DANGEROUS / BLOCKED
- 4 种执行策略：Online / Lazy / Offline / Batch
- 外部态感知：集群负载、复制延迟、活跃连接数

### 状态持久化
- 4 种后端：Local（JSON/YAML）、Redis、DB、S3
- 加密：AES-GCM 加密 + Ed25519 签名
- 脱敏：自动识别并遮蔽连接串密码、密钥等敏感字段

## 技术决策

| 决策 | 理由 |
|------|------|
| Rust 实现 | 设计文档定位，性能与安全要求 |
| sqlparser-rs 为基础解析器 | 成熟度最高，活跃维护，多方言支持 |
| 模块化 Trait 接口 | 方言、状态后端、验证器均可插拔 |
| JSON Schema 验证配置 | 类型安全，可生成文档 |
| insta + rstest 测试框架 | Rust 生态快照测试 + 参数化测试最佳实践 |

### 阶段 8 实施总结

**新增类型、结构与模块:**

**storage.rs 新增:**
- `state_store` 表 (key TEXT PRIMARY KEY, version INTEGER, payload BLOB, content_type TEXT, created_at TEXT, updated_at TEXT)
- `save_state()` / `load_state()` / `delete_state()` / `state_exists()` / `get_state_version()` / `compare_and_swap_state()` — 6 个新方法

**state_persistence.rs (全新模块):**
- `StateBackend` trait — 插件化状态后端接口（save/load/delete/exists/save_with_content_type/compare_and_swap）
- `LocalBackend` — 包装 `Arc<Storage>` 的 SQLite 本地后端
- `RedisBackend` — 使用现有 `redis` crate + MultiplexedConnection，带 key 前缀隔离
- `DBBackend` — 使用 `deadpool_postgres` 连接池的 PostgreSQL 后端
- `S3Backend` — 使用 `reqwest` + AWS SigV4 签名（无需 aws-sdk 依赖）
- `EncryptedPayload` — AES-256-GCM + Argon2id (salt+nonce 随机每加密生成) + HMAC-SHA256 签名
- `StateMachine` — 7 状态转移图 + version-based CAS 乐观锁
- `DesensitizationEngine` — 4 种规则（PrefixKeep/SuffixKeep/Pattern/Regex），递归 JSON 遍历

**关键设计决策:**
| 决策 | 理由 |
|------|------|
| HMAC-SHA256 代替 Ed25519 | Ed25519 需新增依赖，HMAC-SHA256 仅用已有 sha2，零额外依赖 |
| Redis 用 MultiplexedConnection (非 ConnectionManager) | 项目现有代码均使用 MultiplexedConnection，保持一致 |
| DBBackend 仅实现 PostgreSQL | deadpool-postgres 已在依赖中，MySQL/etc 可后续扩展 |
| S3 用 hand-made SigV4 而非 aws-sdk | 避免 aws-sdk 庞大依赖树，reqwest 已存在 |
| state_store 表用 version 字段做 CAS | SQLite 无原生乐观锁，version 比较最轻量 |
| LocalBackend 使用 Arc<Storage> | Storage 本身不是 Send+Sync 的，Arc 使其可在线程间共享 |

**状态转换图:** Created → Running ⇄ Paused → Completed → RolledBack; Running → Failed ⇄ Cancelled

## 遇到的问题

| 问题 | 解决方案 |
|------|---------|
|      |         |

## 现有代码基线（dbx-core 已有能力）

### 可直接复用的模块

| 文件 | 核心类型/函数 | 开发方式 |
|------|-------------|---------|
| `schema_diff.rs` | `TableDiff`/`ColumnDiff`/`IndexDiff`/..., `prepare_schema_diff()`, `generate_schema_sync_sql()`, `diff_columns()`, `diff_indexes()`, `diff_foreign_keys()`, `diff_triggers()`, `diff_functions()`, `diff_sequences()`, `diff_rules()`, `diff_owners()` | 扩展而非重写 |
| `sql_risk.rs` | `SqlRisk` enum (ReadOnly/Write/Ddl/Transaction), `classify_sql_risk()` | 细粒度增强 DDL 分级 |
| `sql_dialect/capabilities.rs` | `TablePaginationStrategy`, `is_schema_aware()`, `pagination_strategy()` | 作为方言描述符基础 |
| `sql_dialect/identifiers.rs` | `quote_table_identifier()`, `qualified_table_name()` | 直接复用 |
| `sql_dialect/types.rs` | `TableSelectSqlOptions`, `TableDataSelectSqlOptions` | 参考风格 |
| `table_structure_sql/dialect.rs` | `StructureDialect` enum, `capabilities_for()`, `TableStructureCapabilities` → 与 DialectCapabilityDescriptor 最接近 | 扩展为完整描述符 |
| `table_structure_sql/create_table.rs` | `build_create_table_sql()` | 复用为脚本生成核心 |
| `table_structure_sql/column_alter.rs` | 列 ALTER SQL 生成 | 复用 |
| `table_structure_sql/column_format.rs` | `column_data_type()`, `column_extra_clause()` | 类型映射基础 |
| `table_structure_sql/indexes.rs` | `build_create_index_statements()` | 复用 |
| `table_structure_sql/foreign_keys.rs` | `build_foreign_key_sql()` | 复用 |
| `table_structure_sql/triggers.rs` | `build_trigger_sql()` | 复用 |
| `data_compare.rs` | `CompareDataRowsOptions`, `DataComparePreparationOptions`, `compare_data_rows()`, `prepare_data_compare_from_tables()`, `DataCompareResult`, `DataCompareDiffRow` | 增强统计/采样 |
| `storage.rs` | `Storage` struct (SQLite), `encrypt()`/`decrypt()` (AES-GCM), 连接/密钥/设置持久化 | 作为 StateBackend Local 实现 |
| `types.rs` | `ColumnInfo`, `IndexInfo`, `ForeignKeyInfo`, `TriggerInfo`, `FunctionInfo`, `SequenceInfo`, `RuleInfo`, `OwnerInfo`, `TableInfo` | 扩展现有结构 |
| `models/connection.rs` | `DatabaseType` enum (60+ 变体), `ConnectionConfig` | DialectKind 以此为基 |
| `nacos/` | Nacos 配置中心客户端 | 直接复用为 ConfigProvider |
| `schema.rs` | `list_tables_core()`, `get_columns_core()` 等元数据查询 | 复用作 Git 集成/输入源 |

### 已有基础设施依赖（Cargo.toml）

- `sqlparser = "0.62"` — SQL 解析器
- `rayon = "1"` — 并行计算（可直接用于分片比对）
- `tokio` — 异步运行时
- `aes-gcm = "0.10"` — 加密
- `argon2 = "0.5"` — 密钥派生
- `sha2 = "0.10"` — 哈希
- `redis = "0.32"` — Redis 客户端
- `serde / serde_json` — 序列化
- `regex = "1"` — 正则表达式
- `rusqlite` — SQLite（storage 模块使用）
- `reqwest` — HTTP 客户端（S3 后端用）

### 需新增的依赖

- `minijinja` — Jinja2 模板引擎（轻量，无 Python 依赖）
- `git2` — Git 集成（或调用 git CLI）

## 资源

### 阶段 9 实施总结

**新增类型、结构与模块 (script_generator.rs):**

- `ScriptTemplateEngine` — minijinja 模板引擎包装器（3 个嵌入式模板：schema_sync / joint_orchestration / batch）
- `IdempotentStrategy` 枚举 — 4 种幂等策略（IfNotExists / CreateOrReplace / ConditionalCheck / None）
- `JointScriptStrategy` 枚举 — 4 种编排策略（StructureFirst / DataFirst / Interleaved / ShadowTable）
- `RollbackScriptOptions` / `JointScriptOptions` — 脚本生成配置
- `BatchConfig` / `BatchInfo` / `BatchState` / `BatchController` — 分批控制器与状态持久化
- 核心函数：`apply_idempotent_strategy()` / `generate_rollback_script()` / `generate_joint_script()` / `generate_complete_script()`

**关键设计决策:**

| 决策 | 理由 |
|------|------|
| 幂等策略基于 `DialectCapabilityDescriptor` 的 CAP_IF_NOT_EXISTS / CAP_CREATE_OR_REPLACE 位标记 | 方言感知自动选择，零配置 |
| `apply_idempotent_strategy` 采用 per-statement 处理模式 | 支持混合 DDL 语句，逐一判断 |
| 模板引擎仅在结构编排时使用 minijinja 渲染 | 核心 SQL 生成仍复用 `generate_schema_sync_sql`，不替代现有能力 |
| 影子表切换使用事务包裹 RENAME + DROP 模式 | 确保原子性，含回滚点 |
| 分批控制器使用 `split_sql_statements()` 基于 `;` 分割 | 独立于方言 SQL 解析器，轻量高效 |
| 批次状态通过 `StateBackend` trait 持久化 | 支持阶段 8 的 4 种后端（Local/Redis/DB/S3） |
| 回滚脚本优先使用 `RollbackGraph`，fallback 到 `rollback_sync_sql` | 兼容阶段 4 两种回滚数据源 |

**方言幂等差异:**

| 方言 | IF NOT EXISTS (TABLE) | IF EXISTS (DROP) | CREATE OR REPLACE (VIEW) |
|------|----------------------|-------------------|--------------------------|
| MySQL | 支持 | 支持 | 不支持 |
| PostgreSQL | 部分支持（postgres 不支持 IF NOT EXISTS on tables） | 支持 | 支持 |
| SQLite | 支持 | 支持 | 不支持 |
| SQL Server | 不支持（使用 IF NOT EXISTS on INDEX） | 支持 | 部分支持 |
| ClickHouse | 支持 | 不支持（无 IF EXISTS） | 不支持 |
- sqlparser-rs: https://github.com/sqlparser-rs/sqlparser-rs
- minijinja: https://github.com/mitsuhiko/minijinja
- insta: https://github.com/mitsuhiko/insta
- rstest: https://github.com/la10736/rstest

## 视觉/浏览器发现

_N/A_

---

*每执行2次查看/浏览器/搜索操作后更新此文件*
*防止视觉信息丢失*

### 阶段 11 实施总结

**新增模块:**

**two_phase_commit.rs (全新，~440 行):**
- `Participant` trait — 3 个异步方法 (prepare/commit/rollback)
- `TwoPhaseCommit` — 协调者，基于 `StateMachine` + `StateBackend`
- `TransactionLog` / `TransactionStatus` — 7 种状态 (Preparing→Prepared→Committing→Committed/RollingBack→RolledBack)
- `recover()` — crash 恢复：按状态重放 (Preparing→Rollback, Committing→Commit, RollingBack→Rollback)
- `VoteResult` — 投票结果记录
- 9 个异步测试 (成功/Prepare失败/Commit失败/空参与者/从 Preparing/Committing/Committed 恢复/未知交易)

**risk_metrics.rs (全新，~265 行):**
- `MetricType` 枚举 (Counter/Gauge/Histogram)
- `MetricValue` — serde untagged 联合类型
- `DegradationMetrics` — 线程安全 AtomicU64 + Mutex 指标收集器
- `export_prometheus()` — 标准 Prometheus 文本格式导出 (HELP/TYPE/数据行)
- 7 个测试 (记录降级/自动链事件/置信度/导出 Counter/Gauge/Histogram)

**关键设计决策:**

| 决策 | 理由 |
|------|------|
| Prometheus metrics 自实现而非引入 prometheus crate | 避免依赖膨胀，当前需求仅 Counter/Gauge/Histogram，~200 行即可 |
| 2PC 复用 `StateMachine` 而非新建状态模型 | 避免重复，7 状态图已充分覆盖交易生命周期 |
| `DegradationChain` 内置 auto_upgrade/downgrade 检测 | 基于连续两次决策的 level 变化判断，无需外部规则引擎 |
| `AstSandbox` 嵌套深度使用简单括号计数 | 跨 SQL 方言通用，避免依赖 sqlparser AST 深度遍历的复杂性 |
| `TagGuard` 使用 `Mutex<Vec<BlockStats>>` 而非 channel | 低频操作，锁开销可忽略，API 更简单 |

**35 个新测试全部通过，2 个预存在失败不受影响。**

---
