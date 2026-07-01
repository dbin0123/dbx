# Plan-Orchestrate Result

**Plan**: `需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md`
**Lang**: unknown (polyglot tie — TS/Rust/Vue/Java; none >60%)
**ECC mode**: legacy (warning: could not detect ECC install; defaulting to legacy form)
**Steps**: 12
**Scope**: all

## Steps overview

| # | Title | Tags | Chain |
|---|---|---|---|
| 1 | 方言适配体系增强 (基于 sql_dialect/ + table_structure_sql/dialect.rs) | refactor, db | `architect,refactor-cleaner,code-reviewer,database-reviewer` |
| 2 | 五层配置继承体系 (全新模块 config/) | design, impl | `planner,architect,tdd-guide,code-reviewer` |
| 3 | SQL 解析与上下文补全增强 (基于 sqlparser + schema.rs) | impl, db | `tdd-guide,code-reviewer,database-reviewer` |
| 4 | 差异计算引擎增强 (核心，改造 schema_diff.rs) | impl, db | `tdd-guide,code-reviewer,database-reviewer` |
| 5 | 数据验证器增强 (基于 data_compare.rs) | impl, test | `tdd-guide,code-reviewer,e2e-runner` |
| 6 | 状态校准器与智能基线重置 (全新模块) | design, impl | `planner,architect,tdd-guide,code-reviewer` |
| 7 | 在线安全评估器增强 (基于 sql_risk.rs) | impl, security | `tdd-guide,code-reviewer,security-reviewer` |
| 8 | 状态持久化层增强 (基于 storage.rs) | impl, db, security | `tdd-guide,code-reviewer,database-reviewer,security-reviewer` |
| 9 | 脚本生成器增强 (基于 schema_diff + table_structure_sql) | impl, db | `tdd-guide,code-reviewer,database-reviewer` |
| 10 | 企业级配置治理与集成 (全新) | design, impl | `planner,architect,tdd-guide,code-reviewer` |
| 11 | 生产级风险控制防御 (全新) | impl, security | `tdd-guide,code-reviewer,security-reviewer` |
| 12 | 集成测试与端到端验证 | test | `tdd-guide,e2e-runner` |

---

## Step 1 — 方言适配体系增强

**Intent**: Upgrade existing `TableStructureCapabilities` (table_structure_sql/dialect.rs) and `sql_dialect/` into a full `DialectCapabilityDescriptor` with type mapping matrix; implement `TypeInferenceEngine` for cross-dialect type inference; reuse `column_data_type()` and `format_default_for_sql()`. Add dialect self-check CLI entry.
**Tags**: refactor, db
**Chain rationale**: `architect` plans the upgrade path from existing `TableStructureCapabilities`; `refactor-cleaner` rewrites the capability struct into the V4 descriptor form; `code-reviewer` gates idiomatic Rust; `database-reviewer` validates type mapping matrix correctness across dialects.

```bash
/orchestrate custom "architect,refactor-cleaner,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-1] Upgrade existing TableStructureCapabilities (table_structure_sql/dialect.rs) and sql_dialect/ into DialectCapabilityDescriptor + TypeMappingMatrix; implement TypeInferenceEngine trait for cross-dialect type inference; add dialect self-check CLI. Reuse column_data_type() and format_default_for_sql(). Acceptance: 1) All existing dialect capability tests still pass; 2) DialectKind ↔ DatabaseType conversion covers 10+ core dialects; 3) MySQL→PG→MySQL type mapping roundtrip preserves semantic equivalence."
```

---

## Step 2 — 五层配置继承体系

**Intent**: Create new `config/` module in dbx-core with 5-layer config hierarchy (Global→Team→Project→Env→Task), expression engine (`${env:VAR}`, `${ref:path}`, `${eval:expr}`), business tag strict anti-penetration mechanism with allowlist, and trace ring buffer. Integrate into existing `connection_config()` call chain.
**Tags**: design, impl
**Chain rationale**: `planner` decomposes the layer merging algorithm and tag policy; `architect` designs the config tree data structures; `tdd-guide` implements the config layer merge/expression parser/tag validator with tests; `code-reviewer` gates correctness and edge cases.

```bash
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-2] Create new config/ module with 5-layer hierarchy (Global→Team→Project→Env→Task), expression engine for \${env:VAR}/\${ref:path}/\${eval:expr}, business tag strict anti-penetration with allowlist, and TraceRingBuffer. Integrate into existing connection_config() path. Acceptance: 1) 5-layer merge priority correct across partial-overlap scenarios; 2) All 3 expression syntax forms resolve with env fallback; 3) Tag strict mode blocks cross-layer penetration and reports blocked path."
```

---

## Step 3 — SQL 解析与上下文补全增强

**Intent**: Add `.meta.json` companion metadata file reader; implement `InputResolver` for mixed input (DDL files + metadata); add `GitDiffScanner` using git CLI to detect changed SQL between commits; implement `AstTransmitFilter` using sqlparser Visitor to whitelist Table/Column/Index/Constraint/View nodes only. Inject into existing `prepare_schema_diff` call chain without breaking its API.
**Tags**: impl, db
**Chain rationale**: `tdd-guide` implements meta-reader, input resolver, git scanner, and AST filter with tests; `code-reviewer` ensures correctness and thread safety; `database-reviewer` validates SQL parsing edge cases across dialects.

```bash
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-3] Add .meta.json companion metadata reader; implement InputResolver for mixed DDL+meta input; add GitDiffScanner for commit-to-commit SQL file detection; implement AstTransmitFilter via sqlparser Visitor whitelisting Table/Column/Index/Constraint/View nodes. Inject into prepare_schema_diff chain. Acceptance: 1) Mixed input produces same SchemaDiffPreparation as pure DDL; 2) AST filter blocks function bodies and trigger bodies; 3) Git diff detects changed .sql files between 2 commits."
```

---

## Step 4 — 差异计算引擎增强

**Intent**: Extend existing `prepare_schema_diff()` with bidirectional Diff + rollback graph generation; add `DependencyGraph` DAG for FK/view topology with topological sort; implement rename detection (column Jaccard + type similarity) and batch naming pattern matching; add dialect-aware type compatibility scoring; shard-parallel diff using rayon. Keep `TableDiff.diff_type` as String for backward compatibility.
**Tags**: impl, db
**Chain rationale**: `tdd-guide` implements the core bidirectional diff, dependency graph, rename detection, and sharded execution with tests; `code-reviewer` validates backward compatibility of the extended TableDiff struct; `database-reviewer` ensures the dependency graph correctly captures FK/view dependencies.

```bash
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-4] Extend prepare_schema_diff() with bidirectional Diff + rollback graph; add DependencyGraph DAG with topological sort; implement rename detection (Jaccard + type similarity) and batch naming patterns; add type compatibility scoring; shard-parallel diff via rayon. Keep TableDiff.diff_type as String for backward compat. Acceptance: 1) Forward∘Rollback = Identity for all test cases; 2) Rename detection >80% similarity accuracy; 3) Sharded diff output identical to single-threaded output."
```

---

## Step 5 — 数据验证器增强

**Intent**: Extend existing `data_compare.rs` with statistical pre-check and automatic downgrade chain (full scan→sampling→skip+risk tag); implement layered extreme-value sampling (random, extreme, stratified, recent-change); add confidence interval calculation; implement structure-data joint correction orchestration with schema-first and data-first strategies.
**Tags**: impl, test
**Chain rationale**: `tdd-guide` implements pre-check/downgrade/sampling/confidence logic with tests; `code-reviewer` validates edge cases in sampling statistics; `e2e-runner` verifies the downgrade chain and joint correction plan through integration scenarios.

```bash
/orchestrate custom "tdd-guide,code-reviewer,e2e-runner" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-5] Extend data_compare.rs with statistical pre-check and auto-downgrade chain (full→sample→skip+risk); implement layered extreme-value sampling (random/extreme/stratified/recent-change); add confidence interval to CompareDataRowsResult; implement structure-data joint correction orchestration. Acceptance: 1) Downgrade triggers at 10/1K/10M row thresholds correctly; 2) Sampling produces statistically representative subsets; 3) Joint correction plan correctly interleaves schema DDL and data migration steps."
```

---

## Step 6 — 状态校准器与智能基线重置

**Intent**: Create new `state_calibrator/` module with `StateSnapshot`, 3-way reconciliation (Baseline↔Source↔Target), semantic fingerprint (DDL normalization→SHA256 using existing `normalize_definition()`), drift detection with pseudo-drift filtering, and smart baseline rebase with conflict handling and history.
**Tags**: design, impl
**Chain rationale**: `planner` defines the reconciliation algorithm; `architect` designs the snapshot/rebase data model; `tdd-guide` implements the 3-way merge, fingerprint, drift detection, and rebase with tests; `code-reviewer` gates correctness of conflict detection.

```bash
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-6] Create state_calibrator/ module with StateSnapshot, 3-way reconciliation (Baseline↔Source↔Target), semantic fingerprint (normalize_definition→SHA256), drift detection with pseudo-drift filter, smart rebase with conflict handling. Acceptance: 1) 3-way merge handles all 9 combination cases; 2) Equivalent DDLs produce identical semantic fingerprints; 3) Rebase with conflicting changes marks conflicts correctly."
```

---

## Step 7 — 在线安全评估器增强

**Intent**: Extend existing `SqlRisk` (ReadOnly/Write/Ddl/Transaction) with fine-grained `DdlRiskLevel` (Safe/Caution/Dangerous/Blocked) for each DDL subtype; implement execution strategy routing (Online/Lazy/Offline/Batch) based on risk + table size + cluster load; add external-state awareness (replication lag, connection count); generate ImpactReport with lock scope and time estimation. Keep backward compatibility with existing `SqlRisk` enum and `classify_sql_risk()`.
**Tags**: impl, security
**Chain rationale**: `tdd-guide` implements risk level classification, strategy router, external-state collector, and impact report generator with tests; `code-reviewer` gates correctness; `security-reviewer` validates that risk levels follow industry best-practice DDL danger classification.

```bash
/orchestrate custom "tdd-guide,code-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-7] Extend existing SqlRisk with DdlRiskLevel (Safe/Caution/Dangerous/Blocked); implement execution strategy router (Online/Lazy/Offline/Batch) from risk+table size+cluster load; add external-state awareness (replication lag, connections); generate ImpactReport. Keep SqlRisk backward compat. Acceptance: 1) All existing SqlRisk classifications unchanged; 2) DDL risk levels correctly map each DDL type; 3) Strategy router picks Online for low-risk ALTER, Blocked for DROP TABLE."
```

---

## Step 8 — 状态持久化层增强

**Intent**: Extend existing SQLite `Storage` into pluggable `StateBackend` trait with Local (SQLite reuse), Redis, DB, and S3 implementations; implement AES-256-GCM encryption + Ed25519 signing using existing `aes-gcm`/`argon2` dependencies; add optimistic concurrency control (CAS) for dual-state-machine transitions; implement state desensitization for sensitive connection fields.
**Tags**: impl, db, security
**Chain rationale**: `tdd-guide` implements the StateBackend trait and all 4 backends with tests; `code-reviewer` validates trait design and error handling; `database-reviewer` reviews Redis/DB/S3 backend correctness; `security-reviewer` gates encryption/signing/desensitization implementation.

```bash
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-8] Extend SQLite Storage into pluggable StateBackend trait with Local/Redis/DB/S3 backends; implement AES-256-GCM + Ed25519 using existing deps; add CAS optimistic concurrency; implement state desensitization. Reuse storage.rs encrypt/decrypt as local. Acceptance: 1) All 4 backends pass CRUD roundtrip; 2) Tampered encrypted+signature payload is rejected; 3) CAS prevents concurrent overwrites."
```

---

## Step 9 — 脚本生成器增强

**Intent**: Layer Jinja2 dialect-aware template engine (minijinja) on top of existing `generate_schema_sync_sql()` and `build_create_table_sql()`; add adaptive idempotency (IF NOT EXISTS/CREATE OR REPLACE/conditional check); implement rollback script generator from RollbackGraph; add batch resumption with savepoint; generate pre/post validation check scripts per dialect.
**Tags**: impl, db
**Chain rationale**: `tdd-guide` implements templating, idempotency injection, rollback generation, and batch resumption with tests; `code-reviewer` gates output correctness; `database-reviewer` validates dialect-specific idempotency and rollback logic.

```bash
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-9] Layer minijinja templates on top of generate_schema_sync_sql() and build_create_table_sql(); add adaptive idempotency (IF NOT EXISTS/CREATE OR REPLACE); implement rollback script generator from RollbackGraph; add batch resumption with savepoints; generate pre/post validation scripts. Acceptance: 1) Template output matches existing generate_schema_sync_sql() output; 2) Rollback after forward scripts restores original state; 3) Batch resumption continues from correct offset after simulated crash."
```

---

## Step 10 — 企业级配置治理与集成

**Intent**: Define `ConfigProvider` trait and implement Apollo, Nacos (reuse existing `nacos/` module), and Consul adapters; implement config change audit log with version management and rollback command; implement canary release engine with progressive rollout (1%→10%→50%→100%) and pause/resume/rollback controls. Integrate into existing `connection_config()` path.
**Tags**: design, impl
**Chain rationale**: `planner` decomposes the config provider integration and canary release algorithm; `architect` designs the ConfigProvider chain and version management data model; `tdd-guide` implements adapters and canary engine with tests; `code-reviewer` gates correctness and error handling.

```bash
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-10] Define ConfigProvider trait with Apollo/Nacos (reuse existing nacos/ module)/Consul adapters; implement config audit log with version management and rollback command; implement canary engine with progressive rollout (1→10→50→100%) and pause/resume/rollback controls. Acceptance: 1) ConfigProvider chain resolves from highest-priority provider; 2) Rollback to version N restores exact state; 3) Canary engine correctly matches target instances by database/tag pattern."
```

---

## Step 11 — 生产级风险控制防御

**Intent**: Implement dual-phase 2PC (prepare→commit/rollback) with crash recovery using state backend; add strict tag breach termination at config read→merge→policy-decision injection points; harden AST whitelist from Phase 3 into isolation sandbox; implement fully automatic validation downgrade chain with Prometheus metrics; implement dependency graph multi-layer coverage scoring (direct→indirect→composite, weighted formula).
**Tags**: impl, security
**Chain rationale**: `tdd-guide` implements 2PC coordinator, tag firewall, AST sandbox, and coverage scoring with tests; `code-reviewer` validates correctness of distributed protocols; `security-reviewer` gates the tag breach termination and AST sandbox isolation mechanisms.

```bash
/orchestrate custom "tdd-guide,code-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-11] Implement dual-phase 2PC (prepare→commit/rollback) with crash recovery; add strict tag breach termination at config read→merge→policy-decision points; harden AST whitelist into sandbox; implement auto validation downgrade chain with Prometheus metrics; implement dep-graph multi-layer coverage scoring (direct→indirect→composite weighted). Acceptance: 1) 2PC survives coordinator crash and auto-recovers; 2) Tag breach terminates with logged path; 3) Coverage score <70% blocks execution, >90% passes."
```

---

## Step 12 — 集成测试与端到端验证

**Intent**: Write backward-compatibility regression tests ensuring all existing `SchemaDiffPreparation`/`SqlRisk`/`generate_schema_sync_sql` outputs remain unchanged; implement cross-dialect full-chain integration tests (MySQL→PG, PG→SQLite); implement bidirectional diff E2E tests (create/delete/modify/rename); benchmark 1000-table diff performance pre/post enhancement; verify Tauri command and dbx-web API contracts.
**Tags**: test
**Chain rationale**: `tdd-guide` writes the regression and integration test suite; `e2e-runner` orchestrates the full-chain end-to-end tests and performance benchmarks. Test steps are exempt from reviewer requirement per Phase 2 rule 10.

```bash
/orchestrate custom "tdd-guide,e2e-runner" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-12] Write backward-compat regression tests for SchemaDiffPreparation/SqlRisk/generate_schema_sync_sql; implement MySQL→PG and PG→SQLite full-chain integration tests; implement bidirectional diff E2E tests (create/delete/modify/rename); benchmark 1000-table diff pre/post enhancement; verify Tauri command and dbx-web API contracts. Acceptance: 1) All existing tests pass unmodified; 2) E2E diff→script→apply→verify completes for 3 dialect pairs; 3) 1000-table diff <5s parallel vs baseline."
```

---

## Batch execution

```bash
/orchestrate custom "architect,refactor-cleaner,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-1] Upgrade existing TableStructureCapabilities (table_structure_sql/dialect.rs) and sql_dialect/ into DialectCapabilityDescriptor + TypeMappingMatrix; implement TypeInferenceEngine trait for cross-dialect type inference; add dialect self-check CLI. Reuse column_data_type() and format_default_for_sql(). Acceptance: 1) All existing dialect capability tests still pass; 2) DialectKind ↔ DatabaseType conversion covers 10+ core dialects; 3) MySQL→PG→MySQL type mapping roundtrip preserves semantic equivalence."
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-2] Create new config/ module with 5-layer hierarchy (Global→Team→Project→Env→Task), expression engine for \${env:VAR}/\${ref:path}/\${eval:expr}, business tag strict anti-penetration with allowlist, and TraceRingBuffer. Integrate into existing connection_config() path. Acceptance: 1) 5-layer merge priority correct across partial-overlap scenarios; 2) All 3 expression syntax forms resolve with env fallback; 3) Tag strict mode blocks cross-layer penetration and reports blocked path."
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-3] Add .meta.json companion metadata reader; implement InputResolver for mixed DDL+meta input; add GitDiffScanner for commit-to-commit SQL file detection; implement AstTransmitFilter via sqlparser Visitor whitelisting Table/Column/Index/Constraint/View nodes. Inject into prepare_schema_diff chain. Acceptance: 1) Mixed input produces same SchemaDiffPreparation as pure DDL; 2) AST filter blocks function bodies and trigger bodies; 3) Git diff detects changed .sql files between 2 commits."
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-4] Extend prepare_schema_diff() with bidirectional Diff + rollback graph; add DependencyGraph DAG with topological sort; implement rename detection (Jaccard + type similarity) and batch naming patterns; add type compatibility scoring; shard-parallel diff via rayon. Keep TableDiff.diff_type as String for backward compat. Acceptance: 1) Forward∘Rollback = Identity for all test cases; 2) Rename detection >80% similarity accuracy; 3) Sharded diff output identical to single-threaded output."
/orchestrate custom "tdd-guide,code-reviewer,e2e-runner" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-5] Extend data_compare.rs with statistical pre-check and auto-downgrade chain (full→sample→skip+risk); implement layered extreme-value sampling (random/extreme/stratified/recent-change); add confidence interval to CompareDataRowsResult; implement structure-data joint correction orchestration. Acceptance: 1) Downgrade triggers at 10/1K/10M row thresholds correctly; 2) Sampling produces statistically representative subsets; 3) Joint correction plan correctly interleaves schema DDL and data migration steps."
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-6] Create state_calibrator/ module with StateSnapshot, 3-way reconciliation (Baseline↔Source↔Target), semantic fingerprint (normalize_definition→SHA256), drift detection with pseudo-drift filter, smart rebase with conflict handling. Acceptance: 1) 3-way merge handles all 9 combination cases; 2) Equivalent DDLs produce identical semantic fingerprints; 3) Rebase with conflicting changes marks conflicts correctly."
/orchestrate custom "tdd-guide,code-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-7] Extend existing SqlRisk with DdlRiskLevel (Safe/Caution/Dangerous/Blocked); implement execution strategy router (Online/Lazy/Offline/Batch) from risk+table size+cluster load; add external-state awareness (replication lag, connections); generate ImpactReport. Keep SqlRisk backward compat. Acceptance: 1) All existing SqlRisk classifications unchanged; 2) DDL risk levels correctly map each DDL type; 3) Strategy router picks Online for low-risk ALTER, Blocked for DROP TABLE."
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-8] Extend SQLite Storage into pluggable StateBackend trait with Local/Redis/DB/S3 backends; implement AES-256-GCM + Ed25519 using existing deps; add CAS optimistic concurrency; implement state desensitization. Reuse storage.rs encrypt/decrypt as local. Acceptance: 1) All 4 backends pass CRUD roundtrip; 2) Tampered encrypted+signature payload is rejected; 3) CAS prevents concurrent overwrites."
/orchestrate custom "tdd-guide,code-reviewer,database-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-9] Layer minijinja templates on top of generate_schema_sync_sql() and build_create_table_sql(); add adaptive idempotency (IF NOT EXISTS/CREATE OR REPLACE); implement rollback script generator from RollbackGraph; add batch resumption with savepoints; generate pre/post validation scripts. Acceptance: 1) Template output matches existing generate_schema_sync_sql() output; 2) Rollback after forward scripts restores original state; 3) Batch resumption continues from correct offset after simulated crash."
/orchestrate custom "planner,architect,tdd-guide,code-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-10] Define ConfigProvider trait with Apollo/Nacos (reuse existing nacos/ module)/Consul adapters; implement config audit log with version management and rollback command; implement canary engine with progressive rollout (1→10→50→100%) and pause/resume/rollback controls. Acceptance: 1) ConfigProvider chain resolves from highest-priority provider; 2) Rollback to version N restores exact state; 3) Canary engine correctly matches target instances by database/tag pattern."
/orchestrate custom "tdd-guide,code-reviewer,security-reviewer" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-11] Implement dual-phase 2PC (prepare→commit/rollback) with crash recovery; add strict tag breach termination at config read→merge→policy-decision points; harden AST whitelist into sandbox; implement auto validation downgrade chain with Prometheus metrics; implement dep-graph multi-layer coverage scoring (direct→indirect→composite weighted). Acceptance: 1) 2PC survives coordinator crash and auto-recovers; 2) Tag breach terminates with logged path; 3) Coverage score <70% blocks execution, >90% passes."
/orchestrate custom "tdd-guide,e2e-runner" "[Plan: 需求问题/2026年6月26日-数据库结构比对与同步工具/task_plan.md#step-12] Write backward-compat regression tests for SchemaDiffPreparation/SqlRisk/generate_schema_sync_sql; implement MySQL→PG and PG→SQLite full-chain integration tests; implement bidirectional diff E2E tests (create/delete/modify/rename); benchmark 1000-table diff pre/post enhancement; verify Tauri command and dbx-web API contracts. Acceptance: 1) All existing tests pass unmodified; 2) E2E diff→script→apply→verify completes for 3 dialect pairs; 3) 1000-table diff <5s parallel vs baseline."
```
