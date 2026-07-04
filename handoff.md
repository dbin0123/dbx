# 🤝 Context Handoff

## Meta
- **exported_at**: 2026-07-04T08:28:27+08:00
- **exported_from**: opencode
- **session_id**: 790504

## Project
- **name**: dbx
- **stack**: Rust, Tauri 2, Vue 3, TypeScript, shadcn-vue, Tailwind CSS, pnpm
- **root**: D:\Developments\jetbrains\workspace\rust\dbx
- **package_manager**: pnpm

## Current Task
在"比较架构"（Schema Diff）页面中增强字段映射规则功能。当源和目标数据库类型不同时（如 MySQL→达梦），支持用户自定义字段类型映射关系，每条映射可控制参数策略（Preserve/Custom），并提供常见数据库对的预设映射。

## Progress
- [x] 后端：添加 ParamStrategy 枚举（Preserve/Custom），更新 FieldMapping 结构体
- [x] 后端：重写 apply_with_params 支持三种策略
- [x] 后端：修复 type_supports_params 中 source_type trim 后切片错位 bug
- [x] 后端：修复 diff_columns_with_compatibility 使用 apply 而非 apply_with_params 的问题
- [x] 后端：5 个单元测试覆盖三种策略及边界情况
- [x] 后端：build.rs 自动扫描 plugins/dialects/*.yaml，编译时嵌入所有方言 YAML
- [x] 后端：register_core_dialects() 在首次使用时将所有方言注册到全局 DialectRegistry
- [x] 后端：type_supports_params 改用 get_by_kind 查找方言（解决注册 key 与 label 不一致问题）
- [x] 后端：移除硬编码的 type_supports_params_hardcoded 回退逻辑
- [x] 前端：更新 FieldMappingEntry 类型（paramStrategy + customParams）
- [x] 前端：创建 fieldMappingPresets.ts（MySQL→达梦/PostgreSQL/Oracle）
- [x] 前端：重写 FieldMappingPanel.vue（预设选择、参数策略下拉、自定义输入）
- [x] 前端：更新 SchemaDiffDialog.vue 序列化格式
- [x] i18n：更新/新增 7 种语言的 fieldMapping 翻译
- [x] 后端测试通过（5/5）

## Active Files
- `crates/dbx-core/build.rs` — 构建脚本，扫描 plugins/dialects/ 目录，生成嵌入所有 YAML 的 core_dialects.rs
- `crates/dbx-core/src/schema_diff.rs` — FieldMapping 结构体、ParamStrategy 枚举、apply/apply_with_params 逻辑、type_supports_params（改用 get_by_kind）、diff_columns_with_compatibility
- `crates/dbx-core/src/sql_dialect/dialect_loader.rs` — register_core_dialects() 函数，编译时嵌入并注册所有方言 YAML
- `crates/dbx-core/src/sql_dialect.rs` — lazy_init() 调用 register_core_dialects()
- `apps/desktop/src/types/schemaDiff.ts` — FieldMappingEntry / FieldMappingParamStrategy 类型定义
- `apps/desktop/src/lib/fieldMappingPresets.ts` — 预设映射定义（MySQL→达梦/PostgreSQL/Oracle）
- `apps/desktop/src/components/diff/FieldMappingPanel.vue` — 字段映射 UI 组件（预设、参数策略、自定义参数）
- `apps/desktop/src/components/diff/SchemaDiffDialog.vue` — Schema Diff 对话框，序列化 fieldMappings
- `apps/desktop/src/components/diff/SchemaDiffConfigStep.vue` — 配置步骤，集成 FieldMappingPanel
- `apps/desktop/src/i18n/locales/en.ts` — 英文翻译（已更新）
- `apps/desktop/src/i18n/locales/zh-CN.ts` — 简体中文翻译（已新增）
- `apps/desktop/src/i18n/locales/zh-TW.ts` — 繁体中文翻译（已新增）
- `apps/desktop/src/i18n/locales/ja.ts` — 日文翻译（已新增）
- `apps/desktop/src/i18n/locales/es.ts` — 西班牙语翻译（已新增）
- `apps/desktop/src/i18n/locales/it.ts` — 意大利语翻译（已新增）
- `apps/desktop/src/i18n/locales/pt-BR.ts` — 葡萄牙语翻译（已新增）
- `docs/superpowers/specs/2026-07-03-field-mapping-rules-design.md` — 设计文档
- `docs/superpowers/plans/2026-07-03-field-mapping-rules.md` — 实现计划

## Blocker
None

## Key Decisions
1. Strip 策略已移除（与 Preserve 在目标不支持参数时行为重复），只保留 Preserve + Custom 两个策略
2. Preserve 策略：目标类型支持参数则保留源类型参数，不支持则自动丢弃
3. Custom 策略：允许用户手动输入自定义参数（如 `(200)` 或 `(10,2)`）
4. 预设映射从 YAML 方言定义推导 + 代码硬编码补充，目前支持 MySQL→达梦/PostgreSQL/Oracle 三组
5. diff_columns_with_compatibility 必须使用 apply_with_params（带参数保留），而非 apply（裸类型映射）
6. **方言 YAML 通过 build.rs 自动扫描 plugins/dialects/ 目录，编译时嵌入并注册到 DialectRegistry**，无需硬编码类型列表
7. type_supports_params 使用 get_by_kind(kind) 查找方言，而非 get(kind.label())，避免因注册 key 不同导致查找失败
8. register_core_dialects() 通过 lazy_init() 在首次解析方言时触发，使用 Once 确保只执行一次

## Environment
- **Node**: v22.20.0
- **Rust**: rustc 1.96.0 / cargo 1.96.0
- **Branch**: cmp
- **Dev command**: `make dev` (Tauri desktop) / `make dev-web` + `make dev-backend` (web)
- **Rust test**: `cargo test -p dbx-core --no-default-features -- <test_name>`
- **Type check**: `npx vue-tsc --noEmit`

## Next Steps
1. 启动 Tauri 开发环境（`make dev`），在浏览器中验证字段映射 UI 是否正常显示
2. 验证预设映射加载和参数策略切换交互
3. 实际执行一次跨库 Schema Diff（如 MySQL→达梦），检查生成的 DDL/SQL 是否正确应用映射规则
4. 考虑是否需要为其他数据库对添加预设映射（如 MySQL→SQL Server、PostgreSQL→达梦 等）

## For the Next AI
- Read all Active Files before doing anything.
- Do NOT change Key Decisions without flagging first.
- This feature is on branch `cmp` — work on that branch.
- Backend tests pass but compilation with DuckDB feature (`cargo test -p dbx-core` without `--no-default-features`) takes very long — use `--no-default-features` for quick iteration.
- The `FieldMapping::apply()` simple method still exists for backward compatibility but should NOT be used in new code — always prefer `apply_with_params()`.
- Core dialect YAMLs are embedded at compile time via `crates/dbx-core/build.rs`. To add a new dialect YAML, just drop the file in `plugins/dialects/` and rebuild — no code changes needed.