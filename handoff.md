# 🤝 Context Handoff

## Meta
- **exported_at**: 2026-07-03T12:30:18+08:00
- **exported_from**: opencode
- **session_id**: b4d9e7

## Project
- **name**: dbx
- **stack**: Rust, Tauri, Vue 3, TypeScript, SQLite, Vite, pnpm
- **root**: D:\Developments\jetbrains\workspace\rust\dbx
- **package_manager**: pnpm

## Current Task
Adding user-customizable field type mapping to Schema Compare (backend Rust + frontend Vue). Users can map source column types to arbitrary target types when source/target DB types differ. Backend applies mappings in SQL generation (`generate_create_table_sql`, `generate_schema_sync_sql`). The frontend `FieldMappingPanel.vue` shows dropdowns populated from `list_dialect_data_types` Tauri command. Recent work added `has_precision: bool` to `DialectType` struct to correctly drive `type_supports_params` for DECIMAL types.

## Progress
- [x] `FieldMapping` Rust struct with `apply` / `apply_with_params` methods — `apply_with_params` preserves params (e.g. `varchar(120)` → `char(120)`)
- [x] `type_supports_params` — queries DialectRegistry for `has_length || has_precision || max_precision`
- [x] `diff_columns_with_compatibility` accepts `field_mappings` and checks user mapping before `matrix.convert_type`
- [x] `generate_create_table_sql` and `generate_schema_sync_sql_inner` both accept `field_mappings`, use `apply_with_params` in `map_type` closures
- [x] Second call in `prepare_schema_diff` (overall sync SQL) changed from 9-arg wrapper to `_inner` with `&options.field_mappings`
- [x] `sourceDialect` auto-detected from `sourceConfig?.db_type` when empty — enables correct cross-dialect type conversion matrix
- [x] Tauri command `list_dialect_data_types(dialect_name)` returning types from DialectRegistry YAML
- [x] Frontend `FieldMappingPanel.vue` calls backend `listDialectDataTypes` with fallback to static `getDataTypeOptions`
- [x] Frontend `api.ts` / `tauri.ts` / `http.ts` all wired for `listDialectDataTypes`
- [x] Frontend types, i18n keys, config step integration, dialog wiring all restored after merge wipes
- [x] Added `has_precision: bool` to `DialectType` struct (with `#[serde(default)]`)
- [x] Added `has_precision: true` to DECIMAL/NUMERIC/NUMBER types in all 30 dialect YAML files (NOT MONEY types)
- [x] Fixed `E0515` borrow error in `type_supports_params` — replaced `and_then`+`find`+`map` with `map`+`any` to avoid returning reference to local `loaded`
- [x] Updated test construction in `dialect_types.rs` with `has_precision: false`
- [x] `cargo check -p dbx-core` passes with 0 errors, 0 new warnings

## Active Files
### Rust backend — Field Mapping core
- `crates/dbx-core/src/schema_diff.rs` — `FieldMapping` struct, `type_supports_params`, all `field_mappings` wiring throughout diff/sql-gen pipeline
- `crates/dbx-core/src/sql_dialect/dialect_yaml.rs:78` — `DialectType` struct now has `has_precision: bool`
- `crates/dbx-core/src/sql_dialect/dialect_types.rs:30` — test construction updated
- `crates/dbx-core/src/sql_dialect/dialect_types.rs` — `list_dialect_type_names()` core function for `list_dialect_data_types`

### Rust backend — Tauri command
- `src-tauri/src/commands/dialect_cmd.rs` — `list_dialect_data_types` Tauri command

### Dialect YAML files (all 30 — added `has_precision: true` to DECIMAL types)
- `plugins/dialects/mysql.yaml` — DECIMAL, FLOAT, DOUBLE
- `plugins/dialects/postgresql.yaml` — NUMERIC(alias DECIMAL)
- `plugins/dialects/sqlserver.yaml` — DECIMAL(alias NUMERIC)
- `plugins/dialects/oracle.yaml` — NUMBER
- `plugins/dialects/sqlite.yaml` — NUMERIC(alias DECIMAL)
- `plugins/dialects/clickhouse.yaml` — Decimal
- `plugins/dialects/dameng.yaml` — NUMERIC(alias DECIMAL, NUMBER)
- `plugins/dialects/access.yaml`, `databend.yaml`, `doris.yaml`, `duckdb.yaml`, `exasol.yaml`, `firebird.yaml`, `gaussdb.yaml`, `gbase.yaml`, `goldendb.yaml`, `h2.yaml`, `highgo.yaml`, `informix.yaml`, `iris.yaml`, `kingbase.yaml`, `kwdb.yaml`, `oceanbase.yaml`, `opengauss.yaml`, `redshift.yaml`, `rqlite.yaml`, `starrocks.yaml`, `sundb.yaml`, `turso.yaml`, `vastbase.yaml`, `vertica.yaml`, `xugu.yaml`, `yashandb.yaml`

### Frontend
- `apps/desktop/src/components/diff/FieldMappingPanel.vue` — dropdown selects with backend `listDialectDataTypes` + fallback
- `apps/desktop/src/components/diff/SchemaDiffConfigStep.vue` — FieldMappingPanel integration
- `apps/desktop/src/components/diff/SchemaDiffDialog.vue` — `handleFieldMappingsUpdate` + `sourceDialect` auto-detect + `fieldMappings` in `prepareSchemaDiff`
- `apps/desktop/src/lib/tauri.ts` — `listDialectDataTypes` binding
- `apps/desktop/src/lib/api.ts` — forward entry
- `apps/desktop/src/lib/http.ts` — HTTP fallback
- `apps/desktop/src/i18n/locales/en.ts` — `fieldMapping.*` i18n keys

### Fixes for pre-existing compilation errors (previous session)
- `crates/dbx-core/src/lib.rs` — `pub mod document_ops;`
- `crates/dbx-core/src/commands/mod.rs` — `pub mod document_cmd;`
- `crates/dbx-core/src/script_generator.rs` — unused import `BindingResult`
- `crates/dbx-core/src/sql_dialect/dialect_loader.rs` — unused import `YamlValidationError`
- `crates/dbx-core/src/sql_dialect/hot_reload.rs` — unused imports
- `crates/dbx-core/src/sql_dialect/inference.rs` — unused import
- `crates/dbx-core/src/osc_probe.rs` — unused variables

## Blocker
None. `cargo check -p dbx-core` passes with 0 errors (only pre-existing warnings).

## Key Decisions
- `FieldMapping::apply` extracts base type before `(` to match `varchar` against `varchar(120)`
- `FieldMapping::apply_with_params` re-appends source params to mapped target type, but only if the target type supports params (checked via DialectRegistry / `type_supports_params`)
- `type_supports_params` uses `map`+`any` (not `and_then`+`find`) to avoid returning reference to local `loaded` data
- `type_supports_params` checks `has_length || has_precision || max_precision.is_some()` — data-driven from YAML, no hardcoded category checks
- `has_precision: bool` added to `DialectType` (with `#[serde(default)]`) — semantically separate from `has_length` (VARCHAR length ≠ DECIMAL precision)
- DECIMAL/NUMERIC types get `has_precision: true`; MONEY types (same `DECIMAL` category) do NOT — MONEY has fixed precision, no user param
- `sourceDialect` auto-detection uses `sourceConfig?.db_type || undefined` to trigger cross-dialect type conversion matrix; user can still override manually in Options panel
- Second `generate_schema_sync_sql` call (overall SQL assembly) uses `_inner` with `&options.field_mappings` — do NOT revert to 9-arg wrapper which passes `&[]`

## Environment
- **OS**: Windows (win32)
- **Node**: v22.20.0
- **pnpm**: 10.27.0
- **Rust**: 1.96.0
- **Branch**: `cmp`
- **Dev command**: `pnpm tauri dev` (launches Vite + Tauri)
- **Frontend port**: http://localhost:1420
- **Cargo check**: `cargo check -p dbx-core` (or `-p dbx` with features)
- **Dialect YAML dir**: `plugins/dialects/`

## Next Steps
1. Run `pnpm tauri dev` (or `pnpm tauri build`) to verify full pipeline compiles and frontend loads
2. Manual test: MySQL→PostgreSQL compare with `varchar→char` mapping, verify generated SQL has `char(120)`, `INTEGER`, `SMALLINT`, `TIMESTAMP`
3. Test DECIMAL→DECIMAL mapping with precision params preserved (e.g. 10,2)
4. Commit changes to `cmp` branch when ready

## For the Next AI
- Read all Active Files before doing anything.
- Do NOT change Key Decisions without flagging first.
- `type_supports_params` in `schema_diff.rs` must use `map`+`any` — do NOT refactor back to `and_then`+`find`.
- `has_precision: bool` is the correct field for DECIMAL types; `has_length` is only for string types.
- MONEY/SMALLMONEY in DECIMAL category must NOT get `has_precision: true` — they have fixed precision.
- The 30 dialect YAML files all have `has_precision: true` on DECIMAL types — verify with grep before editing any YAML.
- Pre-existing unused import/variable warnings are present in `script_generator.rs`, `dialect_loader.rs`, `hot_reload.rs`, `inference.rs`, `osc_probe.rs` — do not fix them unless explicitly asked.
- For frontend, `listDialectDataTypes` is available through `import * as api from "@/lib/api"`.
- i18n keys for field mapping are under `fieldMapping.*` namespace in `en.ts`.

---

✅ handoff.md written to project root.

Summary:
- Task: Field type mapping for Schema Compare — `has_precision` backend + YAML data
- Next step: Run `pnpm tauri dev` to verify full pipeline
- Blocker: None
