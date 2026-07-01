use serde::{Deserialize, Serialize};
use sqlparser::ast::{AlterTableOperation, ObjectType, Statement};
use sqlparser::dialect::{
    ClickHouseDialect, DuckDbDialect, GenericDialect, MsSqlDialect, MySqlDialect, PostgreSqlDialect, SQLiteDialect,
};
use sqlparser::parser::Parser;

/// SQL risk level for agent tool safety classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SqlRisk {
    /// SELECT, SHOW, DESCRIBE, EXPLAIN, WITH (pure read CTE)
    ReadOnly,
    /// INSERT, UPDATE, DELETE, MERGE, REPLACE, CALL/EXEC
    Write,
    /// CREATE, ALTER, DROP, TRUNCATE, GRANT, REVOKE
    Ddl,
    /// BEGIN, COMMIT, ROLLBACK should not be issued by agent
    Transaction,
}

impl std::fmt::Display for SqlRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqlRisk::ReadOnly => write!(f, "read-only"),
            SqlRisk::Write => write!(f, "write"),
            SqlRisk::Ddl => write!(f, "DDL"),
            SqlRisk::Transaction => write!(f, "transaction"),
        }
    }
}

/// Fine-grained DDL risk level for online safety assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DdlRiskLevel {
    /// Low risk: new objects, cosmetic changes (CREATE TABLE, CREATE INDEX)
    Safe,
    /// Medium risk: metadata-only changes, reversible (ADD COLUMN nullable, GRANT)
    Caution,
    /// High risk: table rebuilds, data loss possible (MODIFY COLUMN, TRUNCATE)
    Dangerous,
    /// Critical risk: irreversible destruction (DROP TABLE, DROP SCHEMA)
    Blocked,
}

impl std::fmt::Display for DdlRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdlRiskLevel::Safe => write!(f, "safe"),
            DdlRiskLevel::Caution => write!(f, "caution"),
            DdlRiskLevel::Dangerous => write!(f, "dangerous"),
            DdlRiskLevel::Blocked => write!(f, "blocked"),
        }
    }
}

/// DDL risk classification result for a single statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdlRiskDetail {
    pub ddl_risk: DdlRiskLevel,
    pub summary: String,
    pub affected_objects: Vec<String>,
}

/// Execution strategy for running DDL changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecStrategy {
    /// Direct online execution; minimal lock (< 1s).
    Online,
    /// Deferred/non-blocking (pt-online-schema-change, gh-ost) style.
    Lazy,
    /// Requires a maintenance window; table-level lock expected.
    Offline,
    /// Batched chunk execution, possibly spanning multiple maintenance slots.
    Batch,
}

impl std::fmt::Display for ExecStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecStrategy::Online => write!(f, "online"),
            ExecStrategy::Lazy => write!(f, "lazy"),
            ExecStrategy::Offline => write!(f, "offline"),
            ExecStrategy::Batch => write!(f, "batch"),
        }
    }
}

/// Lock scope information for a DDL operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub lock_type: String,
    pub object: String,
    pub scope: String,
    pub estimated_duration: String,
}

/// Table size category for heuristic strategy selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableSize {
    Small,
    Medium,
    Large,
    Unknown,
}

/// Comprehensive impact assessment report for SQL execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub overall_risk: SqlRisk,
    pub ddl_risk_level: Option<DdlRiskLevel>,
    pub statement_count: usize,
    pub ddl_details: Vec<DdlRiskDetail>,
    pub estimated_locks: Vec<LockInfo>,
    pub estimated_total_duration: String,
    pub recommended_strategy: ExecStrategy,
    pub warnings: Vec<String>,
    pub requires_maintenance_window: bool,
    pub is_reversible: bool,
}

impl ImpactReport {
    pub fn is_safe_for_online_execution(&self) -> bool {
        matches!(self.recommended_strategy, ExecStrategy::Online)
            && !self.requires_maintenance_window
            && self.warnings.is_empty()
    }
}

/// Normalize database dialect string to a canonical form for sqlparser.
/// Mirrors the logic in `sql_analysis::normalize_dialect`.
fn normalize_dialect(dialect: &str) -> &'static str {
    match dialect.to_ascii_lowercase().as_str() {
        "postgres" | "postgresql" | "redshift" | "opengauss" | "gaussdb" | "highgo" => "postgres",
        "mysql" | "mariadb" | "doris" | "starrocks" | "manticoresearch" | "oceanbase" => "mysql",
        "sqlite" => "sqlite",
        "sqlserver" | "mssql" => "sqlserver",
        "clickhouse" => "clickhouse",
        "duckdb" => "duckdb",
        _ => "generic",
    }
}

/// Resolve dialect string to a sqlparser Dialect trait object.
fn resolve_dialect(dialect: &str) -> Box<dyn sqlparser::dialect::Dialect> {
    match dialect {
        "postgres" => Box::new(PostgreSqlDialect {}),
        "mysql" => Box::new(MySqlDialect {}),
        "sqlite" => Box::new(SQLiteDialect {}),
        "sqlserver" => Box::new(MsSqlDialect {}),
        "clickhouse" => Box::new(ClickHouseDialect {}),
        "duckdb" => Box::new(DuckDbDialect {}),
        _ => Box::new(GenericDialect {}),
    }
}

/// Classify a single SQL statement into a risk level using AST analysis.
fn classify_statement(stmt: &Statement) -> SqlRisk {
    match stmt {
        // Pure reads
        Statement::Query(_) => SqlRisk::ReadOnly,
        Statement::Explain { .. } => SqlRisk::ReadOnly,
        Statement::ExplainTable { .. } => SqlRisk::ReadOnly,

        // Show/Describe variants
        Statement::ShowTables { .. }
        | Statement::ShowColumns { .. }
        | Statement::ShowDatabases { .. }
        | Statement::ShowSchemas { .. }
        | Statement::ShowCreate { .. }
        | Statement::ShowVariables { .. }
        | Statement::ShowStatus { .. }
        | Statement::ShowProcessList { .. } => SqlRisk::ReadOnly,

        // Write operations
        Statement::Insert { .. } | Statement::Update { .. } | Statement::Delete { .. } | Statement::Merge { .. } => {
            SqlRisk::Write
        }

        // DDL operations
        Statement::CreateTable { .. }
        | Statement::CreateView { .. }
        | Statement::CreateIndex { .. }
        | Statement::CreateSchema { .. }
        | Statement::CreateSequence { .. }
        | Statement::CreateRole { .. }
        | Statement::CreateType { .. }
        | Statement::AlterTable { .. }
        | Statement::AlterIndex { .. }
        | Statement::AlterView { .. }
        | Statement::Drop { .. }
        | Statement::Truncate { .. } => SqlRisk::Ddl,

        // Grant/Revoke
        Statement::Grant { .. } | Statement::Revoke { .. } => SqlRisk::Ddl,

        // Transaction control
        Statement::StartTransaction { .. } | Statement::Commit { .. } | Statement::Rollback { .. } => {
            SqlRisk::Transaction
        }

        // Copy (PostgreSQL) 鈥?treat as write
        Statement::Copy { .. } => SqlRisk::Write,

        // Pragma (SQLite/DuckDB) 鈥?conservative: treat as write unless known-safe
        Statement::Pragma { .. } => SqlRisk::Write,

        // Catch-all: conservative write classification
        _ => SqlRisk::Write,
    }
}

/// Classify SQL risk using sqlparser AST analysis.
///
/// If parsing fails (non-standard SQL, non-SQL databases), falls back to
/// keyword-based `query_execution_sql::is_write_sql()`.
///
/// Multi-statement input: returns the highest risk level across all statements.
pub fn classify_sql_risk(sql: &str, dialect: &str) -> Result<SqlRisk, String> {
    let normalized = normalize_dialect(dialect);
    let parser_dialect = resolve_dialect(normalized);

    match Parser::parse_sql(parser_dialect.as_ref(), sql) {
        Ok(stmts) if !stmts.is_empty() => {
            let mut max_risk = SqlRisk::ReadOnly;
            for stmt in &stmts {
                let risk = classify_statement(stmt);
                if risk as u8 > max_risk as u8 {
                    max_risk = risk;
                }
            }
            Ok(max_risk)
        }
        _ => {
            // Fallback: keyword-based classification
            if crate::query_execution_sql::is_write_sql(sql) {
                Ok(SqlRisk::Write)
            } else {
                Ok(SqlRisk::ReadOnly)
            }
        }
    }
}

/// Classify a single DDL statement into a fine-grained `DdlRiskLevel`.
/// Returns `None` if the statement is not DDL.
fn classify_ddl_risk_from_statement(stmt: &Statement) -> Option<(DdlRiskLevel, String)> {
    match stmt {
        // --- Create operations (tuple variants in sqlparser 0.62) ---
        Statement::CreateTable(ct) => Some((DdlRiskLevel::Safe, format!("CREATE TABLE {}", ct.name))),
        Statement::CreateView(cv) => Some((DdlRiskLevel::Safe, format!("CREATE VIEW {}", cv.name))),
        Statement::CreateIndex(ci) => {
            let desc = ci.name.as_ref().map(|n| n.to_string()).unwrap_or_else(|| "index".to_string());
            Some((DdlRiskLevel::Safe, format!("CREATE INDEX {}", desc)))
        }
        Statement::CreateRole(cr) => {
            let name = cr.names.first().map(|n| n.to_string()).unwrap_or_else(|| String::from("?"));
            Some((DdlRiskLevel::Caution, format!("CREATE ROLE {}", name)))
        }

        // --- Create operations (struct variants) ---
        Statement::CreateSchema { ref schema_name, .. } => {
            Some((DdlRiskLevel::Safe, format!("CREATE SCHEMA {}", schema_name)))
        }
        Statement::CreateSequence { ref name, .. } => Some((DdlRiskLevel::Safe, format!("CREATE SEQUENCE {}", name))),
        Statement::CreateType { ref name, .. } => Some((DdlRiskLevel::Safe, format!("CREATE TYPE {}", name))),

        // --- Alter Table (tuple variant) ---
        Statement::AlterTable(at) => {
            let table = at.name.to_string();
            let op_risks: Vec<(DdlRiskLevel, &str)> = at
                .operations
                .iter()
                .map(|op| match op {
                    AlterTableOperation::AddColumn { column_def, .. } => {
                        (DdlRiskLevel::Caution, column_def.name.value.as_str())
                    }
                    AlterTableOperation::DropColumn { .. } => (DdlRiskLevel::Dangerous, "drop column"),
                    AlterTableOperation::AlterColumn { column_name, .. } => {
                        (DdlRiskLevel::Dangerous, column_name.value.as_str())
                    }
                    AlterTableOperation::RenameColumn { .. } => (DdlRiskLevel::Dangerous, "rename column"),
                    AlterTableOperation::RenameTable { .. } => (DdlRiskLevel::Dangerous, "rename table"),
                    AlterTableOperation::ChangeColumn { old_name, .. } => {
                        (DdlRiskLevel::Dangerous, old_name.value.as_str())
                    }
                    AlterTableOperation::DropConstraint { name: cn, .. } => {
                        let n = cn.to_string();
                        let risk = if n.to_ascii_lowercase().contains("primary") {
                            DdlRiskLevel::Dangerous
                        } else {
                            DdlRiskLevel::Caution
                        };
                        (risk, "drop constraint")
                    }
                    AlterTableOperation::AddConstraint { .. } => (DdlRiskLevel::Caution, "add constraint"),
                    _ => (DdlRiskLevel::Caution, "alter table"),
                })
                .collect();

            let max_risk = op_risks.iter().map(|(r, _)| *r).max().unwrap_or(DdlRiskLevel::Caution);
            let op_desc: Vec<String> = op_risks.iter().map(|(_, s)| s.to_string()).collect();
            Some((max_risk, format!("ALTER TABLE {} [{}]", table, op_desc.join(", "))))
        }

        // --- Alter Index / View (struct variants) ---
        Statement::AlterIndex { ref name, .. } => Some((DdlRiskLevel::Caution, format!("ALTER INDEX {}", name))),

        Statement::AlterView { ref name, .. } => Some((DdlRiskLevel::Caution, format!("ALTER VIEW {}", name))),

        // --- Drop ---
        Statement::Drop { object_type, ref names, .. } => {
            let obj_names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
            let (risk, desc) = match object_type {
                ObjectType::Table => (DdlRiskLevel::Blocked, format!("DROP TABLE {}", obj_names.join(", "))),
                ObjectType::View => (DdlRiskLevel::Caution, format!("DROP VIEW {}", obj_names.join(", "))),
                ObjectType::Index => (DdlRiskLevel::Caution, format!("DROP INDEX {}", obj_names.join(", "))),
                ObjectType::Schema => (DdlRiskLevel::Blocked, format!("DROP SCHEMA {}", obj_names.join(", "))),
                ObjectType::Sequence => (DdlRiskLevel::Safe, format!("DROP SEQUENCE {}", obj_names.join(", "))),
                _ => (DdlRiskLevel::Dangerous, format!("DROP {} {}", object_type, obj_names.join(", "))),
            };
            Some((risk, desc))
        }

        // --- Truncate (tuple variant) ---
        Statement::Truncate(t) => {
            let table_desc = t.table_names.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
            Some((DdlRiskLevel::Dangerous, format!("TRUNCATE {}", table_desc)))
        }

        // --- Grant / Revoke ---
        Statement::Grant { .. } => Some((DdlRiskLevel::Caution, "GRANT".to_string())),

        Statement::Revoke { .. } => Some((DdlRiskLevel::Caution, "REVOKE".to_string())),

        _ => None,
    }
}

/// Classify a single DDL statement and produce a `DdlRiskDetail`.
fn classify_ddl_detail(stmt: &Statement) -> Option<DdlRiskDetail> {
    classify_ddl_risk_from_statement(stmt).map(|(ddl_risk, summary)| {
        let affected = extract_affected_objects(stmt);
        DdlRiskDetail { ddl_risk, summary, affected_objects: affected }
    })
}

/// Extract human-readable affected object names from a DDL statement.
fn extract_affected_objects(stmt: &Statement) -> Vec<String> {
    let mut objects = Vec::new();
    match stmt {
        Statement::CreateTable(ct) => {
            objects.push(ct.name.to_string());
        }
        Statement::AlterTable(at) => {
            objects.push(at.name.to_string());
        }
        Statement::CreateView(cv) => {
            objects.push(cv.name.to_string());
        }
        Statement::AlterView { ref name, .. } => {
            objects.push(name.to_string());
        }
        Statement::CreateIndex(ci) => {
            objects.push(ci.table_name.to_string());
            if let Some(ref n) = ci.name {
                objects.push(n.to_string());
            }
        }
        Statement::Drop { ref names, .. } => {
            for n in names {
                objects.push(n.to_string());
            }
        }
        Statement::Truncate(t) => {
            for n in &t.table_names {
                objects.push(n.to_string());
            }
        }
        Statement::CreateSchema { ref schema_name, .. } => {
            objects.push(schema_name.to_string());
        }
        Statement::AlterIndex { ref name, .. } => {
            objects.push(name.to_string());
        }
        _ => {}
    }
    objects
}

/// Accumulate multiple DDL risk levels.
/// Returns the highest risk level across all operations.
pub fn accumulate_ddl_risk(levels: &[DdlRiskLevel]) -> DdlRiskLevel {
    levels.iter().max().copied().unwrap_or(DdlRiskLevel::Safe)
}

/// Select execution strategy based on DDL risk, table size, and estimated load.
pub fn select_execution_strategy(
    ddl_risk: DdlRiskLevel,
    table_size: TableSize,
    estimated_load_connections: u32,
) -> ExecStrategy {
    use DdlRiskLevel::*;
    use TableSize::*;

    match (ddl_risk, table_size) {
        // Safe operations: always online
        (Safe, _) => ExecStrategy::Online,

        // Caution on small tables: online; medium/large/unknown: lazy
        (Caution, Small) | (Caution, Unknown) => ExecStrategy::Online,
        (Caution, Medium) | (Caution, Large) => ExecStrategy::Lazy,

        // Dangerous on small tables: lazy (play it safe); medium/large: offline
        (Dangerous, Small) => ExecStrategy::Lazy,
        (Dangerous, Medium | Large | Unknown) => ExecStrategy::Offline,

        // Blocked operations: always require maintenance window unless trivial
        (Blocked, Small) => {
            if estimated_load_connections < LOAD_CONNECTIONS_LOW_THRESHOLD {
                ExecStrategy::Offline
            } else {
                ExecStrategy::Batch
            }
        }
        (Blocked, _) => ExecStrategy::Batch,
    }
}

const SMALL_TABLE_ROWS_MAX: u64 = 10_000;
const MEDIUM_TABLE_ROWS_MAX: u64 = 1_000_000;
const LOAD_CONNECTIONS_LOW_THRESHOLD: u32 = 10;

/// Estimate table size category from row count.
pub fn estimate_table_size(row_count: u64) -> TableSize {
    match row_count {
        0..=SMALL_TABLE_ROWS_MAX => TableSize::Small,
        x if x <= MEDIUM_TABLE_ROWS_MAX => TableSize::Medium,
        _ => TableSize::Large,
    }
}

/// Estimate DDL duration based on table size and risk level.
fn estimate_duration(ddl_risk: DdlRiskLevel, table_size: TableSize) -> &'static str {
    use DdlRiskLevel::*;
    use TableSize::*;

    match (ddl_risk, table_size) {
        (Safe, Small | Unknown) => "< 1 second",
        (Safe, Medium) => "1-5 seconds",
        (Safe, Large) => "5-30 seconds",
        (Caution, Small | Unknown) => "< 5 seconds",
        (Caution, Medium) => "5-30 seconds",
        (Caution, Large) => "30 seconds - 5 minutes",
        (Dangerous, Small) => "5-30 seconds",
        (Dangerous, Medium) => "1-10 minutes",
        (Dangerous, Large | Unknown) => "10 minutes - 2 hours",
        (Blocked, Small) => "< 1 minute",
        (Blocked, Medium) => "1-15 minutes",
        (Blocked, Large | Unknown) => "15 minutes - several hours",
    }
}

/// Build lock information for a DDL statement.
fn build_lock_info(stmt: &Statement, table_size: TableSize) -> Vec<LockInfo> {
    let mut locks = Vec::new();
    let duration = estimate_duration(
        classify_ddl_risk_from_statement(stmt).map(|(r, _)| r).unwrap_or(DdlRiskLevel::Caution),
        table_size,
    );

    match stmt {
        Statement::CreateTable(ct) => {
            locks.push(LockInfo {
                lock_type: "metadata".to_string(),
                object: ct.name.to_string(),
                scope: "schema-level".to_string(),
                estimated_duration: duration.to_string(),
            });
        }
        Statement::CreateView(cv) => {
            locks.push(LockInfo {
                lock_type: "metadata".to_string(),
                object: cv.name.to_string(),
                scope: "schema-level".to_string(),
                estimated_duration: duration.to_string(),
            });
        }
        Statement::AlterTable(at) => {
            let has_rebuild = at.operations.iter().any(|op| {
                matches!(
                    op,
                    AlterTableOperation::DropColumn { .. }
                        | AlterTableOperation::AlterColumn { .. }
                        | AlterTableOperation::ChangeColumn { .. }
                )
            });
            locks.push(LockInfo {
                lock_type: if has_rebuild { "table-exclusive (with rebuild)" } else { "shared-no-write" }.to_string(),
                object: at.name.to_string(),
                scope: "table-level".to_string(),
                estimated_duration: duration.to_string(),
            });
        }
        Statement::Drop { ref names, object_type, .. } => {
            for name in names.iter() {
                locks.push(LockInfo {
                    lock_type: if matches!(object_type, ObjectType::Table | ObjectType::Schema) {
                        "exclusive"
                    } else {
                        "metadata"
                    }
                    .to_string(),
                    object: name.to_string(),
                    scope: match object_type {
                        ObjectType::Schema => "database-level".to_string(),
                        _ => "table-level".to_string(),
                    },
                    estimated_duration: duration.to_string(),
                });
            }
        }
        Statement::Truncate(t) => {
            let table_desc = t.table_names.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
            locks.push(LockInfo {
                lock_type: "exclusive".to_string(),
                object: table_desc,
                scope: "table-level".to_string(),
                estimated_duration: duration.to_string(),
            });
        }
        Statement::CreateIndex(ci) => {
            let idx_name = ci.name.as_ref().map(|n| n.to_string()).unwrap_or_else(|| "unnamed".to_string());
            locks.push(LockInfo {
                lock_type: "shared-no-write".to_string(),
                object: format!("{} (on {})", idx_name, ci.table_name),
                scope: "table-level".to_string(),
                estimated_duration: duration.to_string(),
            });
        }
        _ => {
            let detail = classify_ddl_risk_from_statement(stmt);
            if let Some((_, desc)) = detail {
                locks.push(LockInfo {
                    lock_type: "unknown".to_string(),
                    object: desc,
                    scope: "unknown".to_string(),
                    estimated_duration: duration.to_string(),
                });
            }
        }
    }
    locks
}

/// Generate warnings for a DDL statement based on its risk.
fn generate_warnings(ddl_risk: DdlRiskLevel, stmt_detail: &DdlRiskDetail) -> Vec<String> {
    let mut warnings = Vec::new();
    match ddl_risk {
        DdlRiskLevel::Dangerous => {
            warnings.push(format!(
                "{} — operation may cause table rebuild or block concurrent access",
                stmt_detail.summary
            ));
            warnings.push(
                "Consider using a non-blocking migration tool (e.g. pt-online-schema-change, gh-ost)".to_string(),
            );
        }
        DdlRiskLevel::Blocked => {
            warnings.push(format!("{} — destructive operation, data will be permanently lost", stmt_detail.summary));
            warnings.push("Ensure backups are up-to-date and schedule during maintenance window".to_string());
        }
        DdlRiskLevel::Caution => {
            warnings.push(format!("{} — review impact before execution", stmt_detail.summary));
        }
        DdlRiskLevel::Safe => { /* no warnings */ }
    }
    warnings
}

/// Determine if a set of DDL operations is reversible.
fn is_reversible(details: &[DdlRiskDetail]) -> bool {
    details.iter().all(|d| {
        if matches!(d.ddl_risk, DdlRiskLevel::Blocked) {
            return false;
        }
        if d.summary.starts_with("TRUNCATE") {
            return false;
        }
        true
    })
}

/// Full impact analysis of SQL statements.
///
/// Parses SQL, classifies each statement, and produces a comprehensive
/// `ImpactReport` with risk assessment, lock analysis, strategy recommendation,
/// and warnings.
pub fn analyze_sql_impact(
    sql: &str,
    dialect: &str,
    table_size: TableSize,
    estimated_load_connections: u32,
) -> Result<ImpactReport, String> {
    let normalized = normalize_dialect(dialect);
    let parser_dialect = resolve_dialect(normalized);

    let stmts = Parser::parse_sql(parser_dialect.as_ref(), sql).map_err(|e| format!("SQL parse error: {}", e))?;

    if stmts.is_empty() {
        return Err("No SQL statements found".to_string());
    }

    let mut overall_risk = SqlRisk::ReadOnly;
    let mut ddl_details: Vec<DdlRiskDetail> = Vec::new();
    let mut all_risk_levels: Vec<DdlRiskLevel> = Vec::new();
    let mut all_locks: Vec<LockInfo> = Vec::new();
    let mut all_warnings: Vec<String> = Vec::new();

    for stmt in &stmts {
        let risk = classify_statement(stmt);
        if risk > overall_risk {
            overall_risk = risk;
        }

        if let Some(detail) = classify_ddl_detail(stmt) {
            all_risk_levels.push(detail.ddl_risk);
            let lock_info = build_lock_info(stmt, table_size);
            all_locks.extend(lock_info);
            let warnings = generate_warnings(detail.ddl_risk, &detail);
            all_warnings.extend(warnings);
            ddl_details.push(detail);
        }
    }

    let ddl_risk_level = if all_risk_levels.is_empty() { None } else { Some(accumulate_ddl_risk(&all_risk_levels)) };

    let recommended_strategy = match ddl_risk_level {
        Some(rl) => select_execution_strategy(rl, table_size, estimated_load_connections),
        None => ExecStrategy::Online,
    };

    let estimated_total_duration = match ddl_risk_level {
        Some(rl) => estimate_duration(rl, table_size).to_string(),
        None => "< 1 second".to_string(),
    };

    let requires_maintenance_window = matches!(recommended_strategy, ExecStrategy::Offline | ExecStrategy::Batch);

    let reversible = is_reversible(&ddl_details);

    Ok(ImpactReport {
        overall_risk,
        ddl_risk_level,
        statement_count: stmts.len(),
        ddl_details,
        estimated_locks: all_locks,
        estimated_total_duration,
        recommended_strategy,
        warnings: all_warnings,
        requires_maintenance_window,
        is_reversible: reversible,
    })
}

/// Convenience function: classify DDL risk level from SQL text.
/// Returns `None` if no DDL statements are present.
pub fn classify_ddl_risk(sql: &str, dialect: &str) -> Result<Option<DdlRiskLevel>, String> {
    let report = analyze_sql_impact(sql, dialect, TableSize::Unknown, 0)?;
    Ok(report.ddl_risk_level)
}

pub use crate::safety_report::generate_safety_report;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_select_statements() {
        assert_eq!(classify_sql_risk("SELECT * FROM users", "postgres").unwrap(), SqlRisk::ReadOnly);
        assert_eq!(
            classify_sql_risk("SELECT id, name FROM users WHERE active = true", "mysql").unwrap(),
            SqlRisk::ReadOnly
        );
        assert_eq!(classify_sql_risk("SHOW TABLES", "mysql").unwrap(), SqlRisk::ReadOnly);
        assert_eq!(classify_sql_risk("DESCRIBE users", "mysql").unwrap(), SqlRisk::ReadOnly);
        assert_eq!(classify_sql_risk("EXPLAIN SELECT * FROM users", "postgres").unwrap(), SqlRisk::ReadOnly);
    }

    #[test]
    fn classify_cte_read() {
        assert_eq!(
            classify_sql_risk("WITH cte AS (SELECT 1) SELECT * FROM cte", "postgres").unwrap(),
            SqlRisk::ReadOnly
        );
    }

    #[test]
    fn classify_write_statements() {
        assert_eq!(classify_sql_risk("INSERT INTO users VALUES (1)", "postgres").unwrap(), SqlRisk::Write);
        assert_eq!(classify_sql_risk("UPDATE users SET name = 'x'", "postgres").unwrap(), SqlRisk::Write);
        assert_eq!(classify_sql_risk("DELETE FROM users", "postgres").unwrap(), SqlRisk::Write);
    }

    #[test]
    fn classify_ddl_statements() {
        assert_eq!(classify_sql_risk("CREATE TABLE users (id INT)", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("DROP TABLE users", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("ALTER TABLE users ADD COLUMN age INT", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("TRUNCATE TABLE users", "postgres").unwrap(), SqlRisk::Ddl);
    }

    #[test]
    fn classify_transaction_statements() {
        assert_eq!(classify_sql_risk("BEGIN", "postgres").unwrap(), SqlRisk::Transaction);
        assert_eq!(classify_sql_risk("COMMIT", "postgres").unwrap(), SqlRisk::Transaction);
        assert_eq!(classify_sql_risk("ROLLBACK", "postgres").unwrap(), SqlRisk::Transaction);
    }

    #[test]
    fn classify_multi_statement_returns_highest_risk() {
        // SELECT + INSERT = Write
        assert_eq!(classify_sql_risk("SELECT 1; INSERT INTO users VALUES (1)", "postgres").unwrap(), SqlRisk::Write);
    }

    #[test]
    fn classify_fallback_on_parse_error() {
        // Non-standard SQL should fall back to keyword matching
        assert_eq!(classify_sql_risk("SELECT * FROM users", "generic").unwrap(), SqlRisk::ReadOnly);
    }

    #[test]
    fn classify_unknown_statement_is_write() {
        // Statements not explicitly handled should be conservative (Write)
        // This depends on sqlparser's coverage, but we can test the catch-all
        assert_eq!(classify_sql_risk("GRANT SELECT ON users TO admin", "postgres").unwrap(), SqlRisk::Ddl);
    }

    // --- Phase 7: DDL Risk Level Tests ---

    #[test]
    fn ddl_risk_safe_create_table() {
        let report = analyze_sql_impact("CREATE TABLE users (id INT)", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Safe));
        assert_eq!(report.recommended_strategy, ExecStrategy::Online);
        assert!(!report.requires_maintenance_window);
    }

    #[test]
    fn ddl_risk_safe_create_index() {
        let report =
            analyze_sql_impact("CREATE INDEX idx_name ON users (name)", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Safe));
        assert_eq!(report.recommended_strategy, ExecStrategy::Online);
    }

    #[test]
    fn ddl_risk_caution_add_column() {
        let report =
            analyze_sql_impact("ALTER TABLE users ADD COLUMN age INT", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Caution));
        assert_eq!(report.recommended_strategy, ExecStrategy::Online);
    }

    #[test]
    fn ddl_risk_caution_grant() {
        let report = analyze_sql_impact("GRANT SELECT ON users TO admin", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Caution));
    }

    #[test]
    fn ddl_risk_dangerous_drop_column() {
        let report = analyze_sql_impact("ALTER TABLE users DROP COLUMN age", "postgres", TableSize::Medium, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Dangerous));
        assert_eq!(report.recommended_strategy, ExecStrategy::Offline);
        assert!(report.requires_maintenance_window);
    }

    #[test]
    fn ddl_risk_dangerous_truncate() {
        let report = analyze_sql_impact("TRUNCATE TABLE users", "postgres", TableSize::Medium, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Dangerous));
        assert_eq!(report.recommended_strategy, ExecStrategy::Offline);
    }

    #[test]
    fn ddl_risk_blocked_drop_table() {
        let report = analyze_sql_impact("DROP TABLE users", "postgres", TableSize::Large, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Blocked));
        assert_eq!(report.recommended_strategy, ExecStrategy::Batch);
        assert!(!report.is_reversible);
        assert!(report.requires_maintenance_window);
    }

    #[test]
    fn ddl_risk_blocked_drop_schema() {
        let report = analyze_sql_impact("DROP SCHEMA public CASCADE", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Blocked));
        assert!(!report.is_reversible);
    }

    #[test]
    fn ddl_risk_cumulative_max_risk() {
        let levels = vec![DdlRiskLevel::Safe, DdlRiskLevel::Caution, DdlRiskLevel::Dangerous];
        assert_eq!(accumulate_ddl_risk(&levels), DdlRiskLevel::Dangerous);

        let levels = vec![DdlRiskLevel::Safe, DdlRiskLevel::Blocked, DdlRiskLevel::Caution];
        assert_eq!(accumulate_ddl_risk(&levels), DdlRiskLevel::Blocked);
    }

    #[test]
    fn ddl_risk_cumulative_empty() {
        assert_eq!(accumulate_ddl_risk(&[]), DdlRiskLevel::Safe);
    }

    // --- Phase 7: Multi-statement DDL Tests ---

    #[test]
    fn multi_statement_ddl_risk_escalation() {
        let sql = "CREATE TABLE users (id INT); ALTER TABLE users DROP COLUMN name";
        let report = analyze_sql_impact(sql, "postgres", TableSize::Medium, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Dangerous));
        assert_eq!(report.ddl_details.len(), 2);
    }

    #[test]
    fn multi_statement_mixed_read_and_ddl() {
        let sql = "SELECT 1; CREATE INDEX idx_a ON t (a)";
        let report = analyze_sql_impact(sql, "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.overall_risk, SqlRisk::Ddl);
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Safe));
    }

    // --- Phase 7: ExecStrategy Tests ---

    #[test]
    fn strategy_online_for_safe_ops() {
        assert_eq!(select_execution_strategy(DdlRiskLevel::Safe, TableSize::Large, 100), ExecStrategy::Online);
    }

    #[test]
    fn strategy_lazy_for_caution_large_table() {
        assert_eq!(select_execution_strategy(DdlRiskLevel::Caution, TableSize::Large, 50), ExecStrategy::Lazy);
    }

    #[test]
    fn strategy_offline_for_dangerous() {
        assert_eq!(select_execution_strategy(DdlRiskLevel::Dangerous, TableSize::Medium, 0), ExecStrategy::Offline);
    }

    #[test]
    fn strategy_batch_for_blocked() {
        assert_eq!(select_execution_strategy(DdlRiskLevel::Blocked, TableSize::Large, 0), ExecStrategy::Batch);
    }

    // --- Phase 7: Table Size Tests ---

    #[test]
    fn estimate_table_size_small() {
        assert_eq!(estimate_table_size(0), TableSize::Small);
        assert_eq!(estimate_table_size(10_000), TableSize::Small);
    }

    #[test]
    fn estimate_table_size_medium() {
        assert_eq!(estimate_table_size(10_001), TableSize::Medium);
        assert_eq!(estimate_table_size(1_000_000), TableSize::Medium);
    }

    #[test]
    fn estimate_table_size_large() {
        assert_eq!(estimate_table_size(1_000_001), TableSize::Large);
    }

    // --- Phase 7: Impact Report Tests ---

    #[test]
    fn impact_report_contains_locks() {
        let report = analyze_sql_impact("ALTER TABLE users DROP COLUMN age", "postgres", TableSize::Medium, 0).unwrap();
        assert!(!report.estimated_locks.is_empty());
        let lock = &report.estimated_locks[0];
        assert_eq!(lock.object, "users");
        assert!(lock.lock_type.contains("rebuild"));
    }

    #[test]
    fn impact_report_contains_warnings_for_dangerous() {
        let report = analyze_sql_impact("ALTER TABLE users DROP COLUMN age", "postgres", TableSize::Medium, 0).unwrap();
        assert!(!report.warnings.is_empty());
        assert!(report.warnings.iter().any(|w| w.contains("rebuild")));
    }

    #[test]
    fn impact_report_no_warnings_for_safe() {
        let report = analyze_sql_impact("CREATE INDEX idx ON t (c)", "postgres", TableSize::Small, 0).unwrap();
        assert!(report.warnings.is_empty());
        assert!(report.is_safe_for_online_execution());
    }

    #[test]
    fn impact_report_is_reversible_for_add_column() {
        let report =
            analyze_sql_impact("ALTER TABLE users ADD COLUMN age INT", "postgres", TableSize::Small, 0).unwrap();
        assert!(report.is_reversible);
    }

    #[test]
    fn impact_report_not_reversible_for_drop() {
        let report = analyze_sql_impact("DROP TABLE users", "postgres", TableSize::Small, 0).unwrap();
        assert!(!report.is_reversible);
    }

    #[test]
    fn impact_report_truncate_is_dangerous_not_blocked() {
        let report = analyze_sql_impact("TRUNCATE TABLE t", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Dangerous));
        assert!(!report.is_reversible); // truncate removes all rows — not reversible from a data perspective
    }

    // --- Phase 7: Safety Report Generation Tests ---

    #[test]
    fn generate_safety_report_outputs_string() {
        let report =
            analyze_sql_impact("CREATE TABLE t (id INT); DROP TABLE x", "postgres", TableSize::Small, 0).unwrap();
        let text = generate_safety_report(&report);
        assert!(text.contains("SQL Safety Check Report"));
        assert!(text.contains("CREATE TABLE t"));
        assert!(text.contains("DROP TABLE x"));
        assert!(text.contains("blocked"));
    }

    // --- Phase 7: classify_ddl_risk convenience function ---

    #[test]
    fn classify_ddl_risk_convenience() {
        let r = classify_ddl_risk("CREATE TABLE t (id INT)", "postgres").unwrap();
        assert_eq!(r, Some(DdlRiskLevel::Safe));

        let r = classify_ddl_risk("SELECT 1", "postgres").unwrap();
        assert_eq!(r, None);
    }

    // --- Phase 7: Non-DDL input ---

    #[test]
    fn analyze_impact_pure_read() {
        let report = analyze_sql_impact("SELECT * FROM users", "postgres", TableSize::Small, 0).unwrap();
        assert_eq!(report.overall_risk, SqlRisk::ReadOnly);
        assert_eq!(report.ddl_risk_level, None);
        assert!(report.ddl_details.is_empty());
        assert!(report.estimated_locks.is_empty());
        assert!(report.is_safe_for_online_execution());
    }

    // --- Phase 7: Compatibility with existing SqlRisk ---

    #[test]
    fn existing_classify_sql_risk_still_works_for_ddl() {
        // Ensure that the new fine-grained classification doesn't break existing SqlRisk tests
        assert_eq!(classify_sql_risk("CREATE TABLE users (id INT)", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("DROP TABLE users", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("ALTER TABLE users ADD COLUMN age INT", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("TRUNCATE TABLE users", "postgres").unwrap(), SqlRisk::Ddl);
        assert_eq!(classify_sql_risk("GRANT SELECT ON users TO admin", "postgres").unwrap(), SqlRisk::Ddl);
    }

    #[test]
    fn existing_classify_sql_risk_still_works_for_non_ddl() {
        assert_eq!(classify_sql_risk("SELECT * FROM users", "postgres").unwrap(), SqlRisk::ReadOnly);
        assert_eq!(classify_sql_risk("INSERT INTO users VALUES (1)", "postgres").unwrap(), SqlRisk::Write);
    }

    // --- Phase 7: Edge cases ---

    #[test]
    fn analyze_impact_empty_sql() {
        let result = analyze_sql_impact("", "postgres", TableSize::Small, 0);
        assert!(result.is_err());
    }

    #[test]
    fn ddl_risk_order() {
        assert!(DdlRiskLevel::Safe < DdlRiskLevel::Caution);
        assert!(DdlRiskLevel::Caution < DdlRiskLevel::Dangerous);
        assert!(DdlRiskLevel::Dangerous < DdlRiskLevel::Blocked);
    }

    #[test]
    fn ddl_risk_unnamed_drop_column_field() {
        let r = classify_ddl_risk("ALTER TABLE t DROP COLUMN c", "postgres").unwrap();
        assert_eq!(r, Some(DdlRiskLevel::Dangerous));
    }

    #[test]
    fn ddl_risk_truncate_not_reversible_anymore() {
        let report = analyze_sql_impact("TRUNCATE TABLE t", "postgres", TableSize::Small, 0).unwrap();
        assert!(!report.is_reversible);
        assert_eq!(report.ddl_risk_level, Some(DdlRiskLevel::Dangerous));
    }
}
