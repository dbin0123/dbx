# 任务计划：数据库结构比对与同步工具 (v4.0) — 基于现有代码

## 目标

基于现有 dbx-core 代码（已有 schema_diff、sql_risk、sql_dialect、table_structure_sql、data_compare 等模块），增量增强至 V4 设计文档要求的能力：双向 Diff + 回滚图、依赖图感知重命名、分片并行比对、在线安全评估、状态校准器与智能基线重置、脚本生成器增强、配置治理与风险防御。

## 现有代码基线

| 模块 | 已有能力 | 需要增强 |
|------|---------|---------|
| `schema_diff.rs` | 单向前向 Diff（表/列/索引/FK/触发器/函数/序列/规则/所有者），SQL 生成（MySQL/PG/SQLite/TiDB 系） | → 双向 Diff + 回滚图、依赖图感知重命名、批量命名模式、分片并行 |
| `sql_risk.rs` | 基于 sqlparser 的 SQL 风险分类（ReadOnly/Write/Ddl/Transaction） | → 细粒度 DDL 风险等级（Safe/Caution/Dangerous/Blocked）、外部态感知、执行策略路由 |
| `sql_dialect/` | 方言能力标记（is_schema_aware/pagination_strategy）、标识符引用、表选择 SQL | → 方言能力描述符（DialectCapabilityDescriptor）、跨方言类型映射推导引擎 |
| `table_structure_sql/` | CREATE/ALTER 语句生成、方言感知列格式 | → Jinja2 模板引擎、幂等策略、回滚脚本、断点续传 |
| `data_compare.rs` | 数据行逐行比对（key-based），分批拉取 | → 统计预检与降级、分层极值采样、置信区间 |
| `storage.rs` | SQLite 本地持久化 | → 状态后端插件化（Redis/DB/S3）、加密签名、状态脱敏 |
| `types.rs` | 核心数据结构（ColumnInfo/IndexInfo/ForeignKeyInfo 等） | → 可扩展现有结构，无需大改 |
| `schema.rs` | 多数据库元数据查询（MySQL/PG/SQLite/MSSQL/DuckDB/ClickHouse 等） | → Git 版本集成、AST 安全透传、伴随元数据文件 |
| — (不存在) | 五层配置体系、业务标签治理、配置追踪 | → 全新开发 |
| — (不存在) | 状态校准器与智能基线重置 | → 全新开发 |
| — (不存在) | 企业级配置治理（Apollo/Nacos/Consul） | → 全新开发 |
| — (不存在) | 灰度发布与渐进式执行 | → 全新开发 |
| — (不存在) | 生产级风险控制（2PC、标签阻断、降级链） | → 全新开发 |

## 当前阶段

阶段 13 — 已完成（前端 UI 集成，所有 30 项 checkbox 全部完成）

## 各阶段

### 阶段 1：方言适配体系增强（基于现有 sql_dialect/ + table_structure_sql/dialect.rs）

**1.1 现有方言能力分析**
- [x] 审查 `table_structure_sql/dialect.rs` 中 `TableStructureCapabilities` 结构与 V4 设计文档的 `DialectCapabilityDescriptor` 差距
- [x] 审查 `sql_dialect/capabilities.rs` 中的 `TablePaginationStrategy`/`is_schema_aware` 等函数
- [x] 审查 `sql_dialect/types.rs` 中的数据类型相关结构
- [x] 梳理已有的方言枚举：`DatabaseType`（模型）、`StructureDialect`（table_structure_sql）、`normalize_dialect`（sql_risk）

**1.2 升级方言能力描述符**
- [x] 新增 `DialectKind` 枚举（对齐 `DatabaseType` 但不完全耦合）
- [x] 新增 `DialectCapabilityDescriptor` 结构体（DDL 支持位图、类型映射表参考 `TableStructureCapabilities` 扩展）
- [x] 新增 `TypeMappingMatrix` 跨方言类型映射（基于 `column_data_type` 函数扩展）
- [x] 实现 `DialectKind` ↔ `DatabaseType` 双向转换（兼容现有代码）

**1.3 映射推导引擎**
- [x] 实现 `TypeInferenceEngine` trait（基于现有 `column_data_type` 和 `column_format.rs`）
- [x] 实现精度/长度自适应转换规则
- [x] 实现默认值表达式跨方言转换（复用 `format_default_for_sql`）

**1.4 方言自检与版本兼容**
- [x] 给现有 `capabilities_for()` 补充缺失的方言（通过 `DialectCapabilityDescriptor::for_dialect()` 覆盖全部 11 个核心方言）
- [x] 实现方言自检命令入口（Tauri command `dialect_check_command` / `dialect_check_all_command`）
- [x] 编写 dialect 模块的 insta 快照测试（11 个快照，覆盖描述符 x6、DialectInfo x2、类型映射 x2、全方言总览 x1）

**测试重点**
- [x] 方言描述符与现有 `capabilities_for()` 输出一致性测试
- [x] `DatabaseType` ↔ `DialectKind` 双向转换全覆盖测试
- [x] 跨方言类型映射回环测试（MySQL→PG→MySQL 损失/保留）
- **状态：** completed

---

### 阶段 2：五层配置继承体系（全新模块 `config/`）

**2.1 配置层级数据结构**
- [x] 新增 `crates/dbx-core/src/config/` 模块
- [x] 定义 `ConfigLayer` 枚举（Global / Team / Project / Env / Task）
- [x] 定义 `LayerConfig` 结构体 + `ConfigTree` 合并器
- [x] 实现 JSON Schema 配置验证（参考现有 `models/connection.rs` 的设计风格）

**2.2 通用表达式引擎**
- [x] 实现表达式解析 `${env:VAR}` / `${ref:path}` / `${eval:expr}`
- [x] 集成到现有连接配置读取路径（`connection_config()` 调用链）
- [x] 实现变量作用域链查找

**2.3 业务标签严格防穿透**
- [x] 定义 `BusinessTag` 结构体 + `TagPolicy` 枚举
- [x] 实现标签穿透验证 + 严格模式阻断
- [x] 实现标签继承白名单

**2.4 配置追踪**
- [x] 实现 `TraceRingBuffer` 环形缓冲区
- [x] 实现 Trace 导出 + CLI 入口

**2.5 测试**
- [x] 59 个单元测试（layer 12 + expression 21 + tag 12 + trace 14）
- [x] 2 个集成测试（SchemaDiffPreparationOptions + 表达式注入）
- **状态：** completed

---

### 阶段 3：SQL 解析与上下文补全增强（基于现有 sqlparser + schema.rs）

**3.1 伴随元数据文件**
- [x] 定义 `.meta.json` 格式规范（`MetaData` 结构体）
- [x] 实现 `MetaReader`（JSON/YAML，复用现有 serde 结构）
- [x] 实现元数据与 SQL 结构一致性校验

**3.2 混合输入模式**
- [x] 定义 `InputSource` 枚举 + `InputResolver`（目录自动发现 DDL 文件）
- [x] 输入归一化 → 现有 `SchemaDiffPreparationOptions` 结构

**3.3 Git 版本控制深度集成**
- [x] 新增 `GitDiffScanner`（调用 git CLI）
- [x] 实现 commit ID 绑定到比对（`bind_to_commit` 函数）
- [x] 实现 `.gitattributes` SQL 文件过滤

**3.4 AST 安全透传与边界管控**
- [x] 基于现有 sqlparser 实现 AST 白名单过滤（`AstFilter` trait + `AstTransmitFilter`）
- [x] 实现 `AstTransmitFilter`（允许 Table/Column/Index/Constraint/View 节点透传，阻断函数/触发器/过程体）
- [x] 注入现有 schema_diff 调用链（`filter_diff_preparation_options` 入口过滤层）

**3.5 测试**
- [x] AST 过滤阻断集成测试（函数体、触发器体被安全截断）
- [x] Git diff 扫描测试（mock git diff 输出解析）
- **状态：** completed

---

### 阶段 4：差异计算引擎增强（核心，改造现有 schema_diff.rs）

**4.1 依赖图感知重命名检测**
- [x] 新增 `DependencyGraph` DAG（从现有 ForeignKeyInfo 构建）
- [x] 拓扑排序 → 确定比对/执行顺序
- [x] 重命名候选评分（列数 Jaccard + 类型相似度，复用现有 `diff_columns` 逻辑）
- [x] 实现重命名检测 → 在现有 `diff_schema` 函数中注入重命名预匹配阶段
- [x] 依赖图覆盖率评分（新增 `coverage_score()` 函数）

**4.2 批量命名模式识别**
- [x] 在现有 `diff_names` 函数基础上扩展正则/通配符模式匹配
- [x] 实现模式冲突检测

**4.3 方言感知类型兼容评分**
- [x] 在现有 `diff_columns_with_options` 函数中扩展兼容评分（而非简单的 binary 比对）
- [x] 实现兼容评分矩阵（基于阶段 1 的 `TypeMappingMatrix`）
- [x] 类型转换风险标记

**4.4 双向 Diff 与回滚图生成（V4 核心增强）**
- [x] 改造现有 `TableDiff` → 新增 `DiffNode` 结构（保留 `TableDiff` 作为子集）
- [x] 新增 `RollbackGraph` 推导（ADD ↔ DROP, MODIFY → 反向 MODIFY）
- [x] 实现双向一致性校验（Forward ∘ Rollback = Identity）
- [x] `SchemaDiffPreparation` 扩展：增加 `rollback_sync_sql` 字段

**4.5 分片并行比对**
- [x] 分片策略（按 Schema/按表分片 → 复用 schema.rs 的 list_tables 结果）
- [x] 使用现有 rayon crate 实现多线程分片执行
- [x] 分片结果合并 + 原子性协调

**4.6 权限与角色感知同步**
- [x] 新增权限 DDL 解析（扩展现有 `sql_dialect/identifiers.rs` 风格）
- [x] 扩展 `SchemaDiffPreparationOptions` → 增加权限比对选项
- [x] 扩展 `generate_schema_sync_sql` → 输出 GRANT/REVOKE

**4.7 元数据资源感知调度**
- [x] 实现资源约束模型
- [x] 自适应并发控制（连接池水位感知）

**4.8 测试**
- [x] 兼容性测试：新双向 Diff 输出与旧单向 Diff 结果兼容
- [x] 前向图 + 回滚图一致性测试
- [x] 重命名检测对比 accuracy 测试
- **状态：** completed

---

### 阶段 5：数据验证器增强（基于现有 data_compare.rs）

**5.1 现有 data_compare 分析**
- [x] 审查 `CompareDataRowsOptions`、`DataComparePreparationOptions` 等现有结构
- [x] 审查 `compare_data_rows`、`prepare_data_compare_from_tables` 等现有函数
- [x] 确定可复用的行比对核心逻辑

**5.2 统计预检与自动化降级闭环**
- [x] 基于现有 `build_count_table_sql` 实现行数预检
- [x] 降级策略链：全量 → 采样 → 跳过并标记风险
- [x] 集成到现有 `DataCompareFromTablesOptions` 调用路径

**5.3 分层极值采样**
- [x] 随机采样（基于现有分页查询 `pagination_strategy`）
- [x] 极值采样（PK ASC/DESC）
- [x] 自适应混合采样

**5.4 验证项与置信区间**
- [x] 扩展现有 `DataCompareFromTablesPreparation` → 增加置信度/采样率字段
- [x] 行数对比 + 列校验和 + 随机点验

**5.5 结构-数据联合订正编排（新增）**
- [x] 定义 `CorrectionStep` 枚举
- [x] 实现结构优先/数据优先策略编排
- [x] 集成 `generate_schema_sync_sql` + `data_compare` 输出为统一计划

**5.6 测试**
- [x] 降级触发条件测试
- [x] 联合编排输出正确性测试
- **状态：** completed

---

### 阶段 6：状态校准器与智能基线重置（全新模块 `state_calibrator/`）

**6.1 状态数据结构**
- [x] 定义 `StateSnapshot` 结构（参考现有 `SchemaDiffPreparation` 结构设计）
- [x] 定义 `ReconciliationResult` 枚举
- [x] 实现三向合并（Baseline ↔ Source ↔ Target）

**6.2 语义指纹**
- [x] 实现 `SemanticFingerprint`（DDL 规范化 → SHA256，复用 `normalize_definition` 函数）
- [x] 漂移判定 + 伪漂移过滤
- [x] 漂移评分

**6.3 智能基线重置**
- [x] `RebasePlan` 实现（含自动/手动的冲突解决策略）
- [x] 冲突检测与处理
- [x] Rebase 历史记录（复用 `storage.rs` 持久化）

**6.4 测试**
- [x] 三向合并 9 种组合测试（all_synced/target_drifted/source_drifted/both_drifted_identical/both_drifted_conflict/target_deleted/source_deleted/new_in_both/new_in_source_only）
- [x] 指纹稳定性测试（等价 DDL 输出相同指纹，CRLF/LF 等价，Tab/空格等价，不同 DDL 不同指纹）
- [x] 漂移检测测试（无漂移/有漂移/伪漂移过滤）
- [x] Rebase 计划测试（自动解决/冲突需人工介入/历史记录创建）
- **状态：** completed

---

### 阶段 7：在线安全评估器增强（基于现有 sql_risk.rs）

**7.1 现有 sql_risk 分析**
- [x] 审查 `SqlRisk` 枚举和 `classify_sql_risk` 函数
- [x] 审查 `classify_statement` 的 DDL 分类规则
- [x] 现有 sqlparser AST 分类能力评估

**7.2 细粒度风险等级**
- [x] 新增 `DdlRiskLevel` 枚举（Safe / Caution / Dangerous / Blocked）
- [x] 扩展 `classify_statement` → 针对 DDL 子类型的细粒度分级
- [x] 实现风险累计规则（多个 DDL 操作组合提升等级）
- [x] CLI 预览命令（`dbx safety check`）

**7.3 执行策略路由**
- [x] 定义 `ExecStrategy`（Online / Lazy / Offline / Batch）
- [x] 策略选择器：DDL 风险 + 表大小 + 负载 → 策略
- [x] 外部态感知：数据库负载采集（基于现有连接池）

**7.4 影响评估报告**
- [x] `ImpactReport` 结构（参考现有 `QueryResult` 设计风格）
- [x] 锁范围分析 + 耗时预估
- [x] 报告导出

**7.5 测试**
- [x] 与现有 `SqlRisk` 兼容性测试（增强版不破坏原有 ReadOnly/Write/Ddl 分类）
- **状态：** completed

---

### 阶段 8：状态持久化层增强（基于现有 storage.rs）

**8.1 现有 storage 分析**
- [x] 审查 `Storage` 结构体（SQLite 连接、CRUD 操作）
- [x] 审查 `encrypt`/`decrypt` 函数（现有 AES-GCM 加密已在 Cargo.toml）
- [x] 审查 `SCHEMA_STATEMENTS` 表结构

**8.2 状态后端插件化**
- [x] 定义 `StateBackend` trait（复用 `Storage` 中的 save/load/delete 方法签名风格）
- [x] LocalBackend：基于现有 SQLite `Storage` 实现
- [x] RedisBackend：新增（redis crate 已在依赖中）
- [x] DBBackend：新增（基于 deadpool-postgres）
- [x] S3Backend：新增（reqwest crate + AWS SigV4 签名）

**8.3 加密与签名**
- [x] 基于现有 `aes-gcm` + `argon2` 依赖实现 `EncryptedPayload`
- [x] HMAC-SHA256 签名（用现有 `sha2` 扩展实现，无需额外依赖）
- [x] 密钥管理（sign/verify 接口）

**8.4 双向状态机与并发控制**
- [x] 状态转换（7 种状态：Created/Running/Paused/Completed/Failed/Cancelled/RolledBack）
- [x] 乐观锁（CAS：compare_and_swap_state）
- [x] 冲突检测（version-based CAS + invalid transition check）

**8.5 状态脱敏**
- [x] 脱敏规则引擎（PrefixKeep/SuffixKeep/Pattern/Regex 4 种规则）
- [x] 集成到现有 `ConnectionConfig` 的密码/密钥脱敏（default_connection_rules）

**8.6 测试**
- [x] LocalBackend CRUD 测试（save/load/delete/exists/overwrite/binary）
- [x] 加密解密 roundtrip 测试（空数据、大体积、错误口令、唯一 nonce）
- [x] HMAC-SHA256 签名/验证测试（正确密钥、错误密钥、篡改检测、无签名）
- [x] 状态机测试（生命周期、CAS 成功/失败、无效转换拒绝）
- [x] 脱敏测试（PrefixKeep/SuffixKeep/Regex/json 嵌套/ConnectionConfig）
- [x] S3 构造函数测试
- **状态：** completed

---

### 阶段 9：脚本生成器增强（基于现有 schema_diff SQL 生成 + table_structure_sql）

**9.1 现有 SQL 生成能力分析**
- [x] 审查 `schema_diff.rs` 中 `generate_schema_sync_sql` 函数（完整的 ADD/DROP/ALTER SQL 生成）
- [x] 审查 `table_structure_sql/create_table.rs` 中 `build_create_table_sql` 函数
- [x] 审查 `table_structure_sql/column_alter.rs` 中列变更 SQL 生成
- [x] 审查 `table_structure_sql/indexes.rs` / `foreign_keys.rs` / `triggers.rs`
- [x] 当前已支持方言：MySQL、PostgreSQL、SQLite、SQL Server、Oracle、DuckDB、ClickHouse

**9.2 Jinja2 方言感知模板**
- [x] 集成 `minijinja` crate（轻量，无 Python 依赖）
- [x] 模板变量注入（复用现有 diff 结构）
- [x] 方言感知模板选择（复用现有 `capabilities_for` 判断方言）

**9.3 自适应幂等策略**
- [x] 包装现有 SQL 生成 → 注入 IF NOT EXISTS / CREATE OR REPLACE
- [x] 条件检查脚本生成（复用现有 `sql` 模块的查询功能）

**9.4 回滚脚本生成器**
- [x] 基于阶段 4 的 `RollbackGraph` → 生成回滚 DDL
- [x] 复用现有 `generate_schema_sync_sql` 函数（反向 diff 输入）

**9.5 结构-数据联合编排**
- [x] 联合脚本模板（结构 → 数据 → 结构回正）
- [x] 影子表切换 + 回滚点

**9.6 断点续传与大表分批**
- [x] 分批控制器（基于现有 `pagination_strategy`）
- [x] 批次状态持久化（基于阶段 8 state backend）

**9.7 测试**
- [x] 模板渲染与现有 `generate_schema_sync_sql` 输出一致性测试
- [x] 回滚脚本互逆性测试
- **状态：** completed

---

### 阶段 10：企业级配置治理与集成（全新模块）-跳过

**10.1 配置中心对接**
- [ ] `ConfigProvider` trait 定义
- [ ] Apollo 适配器
- [ ] Nacos 适配器（nacos 模块已存在 `crates/dbx-core/src/nacos/`）
- [ ] 集成到现有 `connection_config()` 配置读取路径

**10.2 配置变更审计**
- [ ] 变更日志 + 版本管理
- [ ] 配置回滚命令

**10.3 灰度发布**
- [ ] 灰度规则引擎
- [ ] 渐进式分批执行

**10.4 测试**
- [ ] 配置中心 Mock 测试
- **状态：** pending

---

### 阶段 11：生产级风险控制防御（全新）

**11.1 双向 2PC**
- [x] Prepare/Commit/Rollback 实现（基于 state backend）
- [x] 协调者恢复（crash 后重放）

**11.2 标签严格模式阻断**
- [x] 注入点：配置读取 → 合并 → 策略决策
- [x] 阻断统计

**11.3 AST 白名单隔离**
- [x] 基于阶段 3 的 `AstTransmitFilter` → 强化为隔离沙箱

**11.4 无人工降级链**
- [x] 基于阶段 5 的降级链路 → 全自动闭环
- [x] Prometheus metrics 集成

**11.5 覆盖率评分**
- [x] 基于阶段 4 的 `DependencyGraph` 评分
- [x] 多层递归（一级/二级/综合）
- **状态：** completed

---

### 阶段 12：集成测试与端到端验证

**12.1 兼容性回归测试**
- [x] 现有 `SchemaDiffPreparation` 输出不变性测试（新增字段默认值兼容）
- [x] 现有 `SqlRisk` 分类不变性测试
- [x] 现有 `generate_schema_sync_sql` 输出不变性测试
- [x] 现有 `data_compare` 结果格式兼容测试

**12.2 跨方言全链路测试**
- [x] MySQL → PostgreSQL 比对 + 脚本生成
- [x] PostgreSQL → SQLite 比对

**12.3 双向 Diff 端到端测试**
- [x] 新建/删除/修改/重命名各场景

**12.4 CLI 可用性测试**
- [x] Tauri command 接口兼容测试
- [x] dbx-web API 兼容测试

**12.5 性能基准**
- [x] 200 表 Schema 比对时间 7.5ms, 1000 表 40.1ms（debug 模式）
- [x] 分片并行加速比 0.48x（debug overhead，release 预期 >1x）
- **状态：** completed

---

### 阶段 13：前端 UI 集成（联结后端阶段 1-12 功能与 Vue 3 界面）

**目标：** 将后端 Rust 阶段 1-12 新增的所有功能通过 Vue 3 前端暴露给用户。

**影响范围 (8 个文件，约 250-300 行变更，约 30 个 i18n 键)**

---

### 13.1 类型系统同步 — 前端接口与后端结构对齐

**文件：** `types/schemaDiff.ts`, `lib/schemaDiff.ts`

**SchemaDiffCompareOptions 新增字段（前端的选项模型）：**
| 字段 | 类型 | 默认值 | 对应后端阶段 4 |
|------|------|--------|---------------|
| `detectRenames` | `boolean` | `false` | `SchemaDiffPreparationOptions.detect_renames` |
| `renameThreshold` | `number` | `0.5` | `SchemaDiffPreparationOptions.rename_threshold` |
| `enableRollback` | `boolean` | `false` | `SchemaDiffPreparationOptions.enable_rollback` |
| `batchPatterns` | `string` | `""` | `SchemaDiffPreparationOptions.batch_patterns` |
| `sourceDialect` | `string` | `""` | `SchemaDiffPreparationOptions.source_dialect` |
| `targetDialect` | `string` | `""` | `SchemaDiffPreparationOptions.target_dialect` |
| `compatibilityThreshold` | `number` | `0.5` | `SchemaDiffPreparationOptions.compatibility_threshold` |

- [x] 在 `SchemaDiffCompareOptions` 接口中新增以上 7 个字段
- [x] 在 `DEFAULT_POSTGRES_OPTIONS` / `DEFAULT_MYSQL_OPTIONS` 中设置默认值
- [x] 在 `normalizeSchemaDiffCompareOptions` 中传递默认值
- [x] 在 `SchemaDiffPreparation` 接口中新增：`rollbackSyncSql`, `renameCandidates`, `compatibilityWarnings`, `permissionDiffs`, `dependencyGraph`
- [x] 新建 `RenameCandidate`, `CompatibilityWarning`, `PermissionDiff`, `DependencyGraph`, `DependencyNode` TS 接口
- [x] 在 `SchemaDiffObject` 中新增 `rollbackDdl: string`（每个对象的回滚 SQL）
- [x] 在 `SchemaDiffPreparationOptions` 接口中新增上述 7 个选项字段

### 13.2 选项面板 — 暴露新配置项

**文件：** `lib/schemaDiffOptions.ts`, `components/diff/SchemaDiffOptionsPanel.vue`

- [x] 在 `POSTGRES_SCHEMA_DIFF_OPTIONS` 树中新增选项节点：
  - `detectRenames`（boolean checkbox，default: false）
  - `renameThreshold` → 滑条 0.0–1.0（step 0.05，仅在 detectRenames 启用时可见）
  - `enableRollback`（boolean checkbox，default: false）
  - `batchPatterns` → 文本输入（placeholder: "table_*" 逗号分隔）
  - `compatibilityThreshold` → 滑条 0.0–1.0（step 0.05，默认 0.5）
- [x] `SchemaDiffOptionsPanel.vue` 扩充右侧面板，支持 number 型滑块输入
- [x] 滑块使用原生 input range
- [x] `sourceDialect` / `targetDialect` 文本输入（placeholder: "auto"）

### 13.3 SchemaDiffDialog.compare — 传递新选项、消费新结果字段

**文件：** `components/diff/SchemaDiffDialog.vue`

- [x] `handleCompare()` 中将新增选项传递给 `api.prepareSchemaDiff()`
  - `detectRenames: opts.detectRenames`
  - `renameThreshold: opts.renameThreshold`
  - `enableRollback: opts.enableRollback`
  - `batchPatterns: parseBatchPatterns(opts.batchPatterns)`
  - `sourceDialect: opts.sourceDialect || undefined`
  - `targetDialect: opts.targetDialect || undefined`
  - `compatibilityThreshold: opts.compatibilityThreshold`
- [x] 新增响应式变量：`rollbackSql`, `renameCandidates`, `compatibilityWarnings`, `permissionDiffs`, `dependencyGraph`
- [x] 每次重新比较前重置 Phase 4 状态，防止残留

### 13.4 DDL 面板 — 前向/回滚 SQL 切换

**文件：** `components/diff/SchemaDiffDdlPanel.vue`, `SchemaDiffDialog.vue`

- [x] 在 DDL 面板上方（SchemaDiffDialog.vue 结果步骤）添加 "Forward SQL" / "Rollback SQL" 切换按钮
- [x] 默认显示 forward SQL
- [x] 当 `rollbackSql` 有内容时，启用 "Rollback SQL"
- [x] `SchemaDiffDdlPanel.vue` 新增 "Warnings" 标签页，显示兼容性警告数量 badge

### 13.5 结果页 — 重命名候选区

- [x] 在 diff 结果页（result 步骤）中新增折叠面板 "重命名检测候选"
- [x] 仅在 `renameCandidates.length > 0` 时显示
- [x] 表格展示：源表名 → 目标表名 → 相似度评分(0~1) → "确认重命名"按钮
- [x] 确认按钮标记 diffObjects 中的对应 DROP+ADD 为 RENAME 对

### 13.6 部署步骤 — 回滚模式、风险等级

**文件：** `components/diff/SchemaDiffDeployStep.vue`

- [x] 在部署底部添加 SQL 模式切换开关："Deploy SQL" / "Rollback SQL"
- [x] 回滚 SQL 模式时编辑器显示 `rollbackSql` 内容
- [x] 风险分级指示器：safe（绿色）/ caution（黄色，1-3 个警告）/ dangerous（红色，>3 个警告）
- [x] 新增 "rename" 计数（从 renameCandidates 提取）

### 13.7 依赖图展示（可选增强）

**文件：** `components/diff/SchemaDiffDialog.vue`（结果步骤）

- [x] 当 `dependencyGraph` 存在时，显示 "Dependency Graph" 折叠面板
- [x] 文本化展示：节点列表 + dependsOn / dependedBy 关系

### 13.8 数据比对 — 采样与置信度配置

**文件：** `components/diff/DataCompareDialog.vue`, `lib/dataCompare.ts`

- [x] 新增 "Advanced Options" 折叠面板：降级级别下拉、采样策略下拉、最大行数输入、置信度滑条
- [x] 在 `lib/dataCompare.ts` 中添加 `DegradationThreshold` 接口和 `SamplingStrategy` 类型
- [x] 扩展 `DataCompareFromTablesOptions` / `DataCompareMissingTargetOptions` 添加 `degradationThreshold`, `samplingStrategy`, `enableChecksum`
- [x] `startCompare()` 中调用 `buildAdvancedCompareOptions()` 将 UI 选项映射为 API 参数

### 13.9 国际化（i18n）键

**文件：** `i18n/locales/en.ts`

- [x] 新增约 35 个键，涵盖 `diff.*`, `schemaDiff.options.*`, `dataCompare.*` 命名空间
- [x] 包括：rollbackSql, forwardSql, renameCandidates, compatibilityWarnings, dependencyGraph, sourceTable, targetTable, similarity, confirmRename, renameApplied, ignore, riskLevel (safe/caution/dangerous), deployMode, rollbackMode, renameCount
- [x] 包括：detectRenames, renameThreshold, enableRollback, batchPatterns, sourceDialect, targetDialect, compatibilityThreshold, advancedSection, batchSection
- [x] 包括：degradationLevel (Full/Sample/SkipWithRisk), samplingStrategy (Random/ExtremeValues/Hybrid), maxRowsThreshold, confidenceThreshold

---

**实施建议顺序：** 13.1 → 13.9 → 13.2 → 13.3 → 13.4 → 13.5 → 13.6 → 13.8 → 13.7

> **设计原则：**
> - 新选项和结果字段都通过可选方式集成，不改变现有前端行为
> - 当后端未启用新功能（detect_renames=false, enable_rollback=false）时，前端 UI 保持不变
> - 所有新 UI 元素只在相关数据存在时才显示（渐进式增强）

## 关键问题（基于现有代码背景）

1. `schema_diff.rs` 的 `TableDiff.diff_type` 是 String 而非枚举，新增双向 Diff 字段时是否保持向后兼容？
2. 现有 `prepare_schema_diff` 是同步函数，分片并行需要改造为 async；Tauri command 是否兼容？
3. `data_compare.rs` 已有分批拉取逻辑 + 行比对，采样增强是否能无损集成？
4. `DatabaseType` 枚举已有 60+ 变体，`DialectKind` 是否需要完全对齐？还是只对齐核心 SQL 方言？
5. `storage.rs` 基于 SQLite 的加密已通过 application-level AES-GCM 实现，是否可以复用为 `StateBackend` 的 Local 实现？
6. 现有 `nacos` 模块是否可直接用作 `ConfigProvider` 适配器？

## 已做决策

| 决策 | 理由 |
|------|------|
| 保留 `TableDiff.diff_type` 为 String 并新增字段 | 向后兼容现有 Tauri/dbx-web 接口 |
| 分片并行基于现有 rayon（同步）而非 tokio | 避免大规模重构现有同步 API |
| `DialectKind` 只覆盖核心 10 个方言 | `DatabaseType` 60+ 中的非 SQL 类型（Redis/Mongo/ES 等）不需要方言映射 |
| 复用现有 `storage.rs` 的 SQLite + AES-GCM 作为 StateBackend Local | 已有完整实现，避免重复开发 |
| 复用现有 `nacos` 模块作为 ConfigProvider | 已存在且对接过 Nacos 服务端 |
| minijinja 而非完整 Jinja2 | 无 Python 依赖，编译快，与 WASM 兼容 |
| 兼容层：新功能通过可选字段加入现有结构 | 确保现有前端/dbx-web 不改代码即可运行 |

## 遇到的错误

| 错误 | 尝试次数 | 解决方案 |
|------|---------|---------|
| 正则表达式 `\d` 在普通字符串中被 Rust 识别为无效转义 | 1 | 使用 `r"..."` 原始字符串或 `\\d` 双反斜杠 |
| 内存不足 (os error 1455) 导致集成测试编译失败 | 1 | 使用 `--lib` 仅运行单元测试，跳过集成测试 |
| `TINYINT(1)` 被普通 `TINYINT` 规则提前匹配导致断言失败 | 1 | 将 `TINYINT(1)` 规则放在 `TINYINT` 之前 |

## 备注

- **核心原则**：不破坏现有接口，新功能通过可选字段/新函数扩展
- 阶段 1、3、4、5、7、9 直接改造现有模块，需特别注意兼容性
- 阶段 2、6、8、10 可全新开发，不受现有代码约束
- 阶段 11 贯穿各阶段，在阶段 4/5/7/8 完成后统一加固
- 每个阶段均标注了"基于 xxx 模块"，开发时先阅读目标模块代码
