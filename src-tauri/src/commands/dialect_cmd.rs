use dbx_core::sql_dialect::{dialect_check, dialect_check_all, DialectInfo, DialectKind};

#[tauri::command]
pub fn dialect_check_command(kind: String) -> Result<DialectInfo, String> {
    let db_type = match kind.to_ascii_lowercase().as_str() {
        "mysql" => DialectKind::Mysql,
        "postgres" | "postgresql" => DialectKind::Postgres,
        "sqlite" => DialectKind::Sqlite,
        "duckdb" => DialectKind::DuckDb,
        "sqlserver" | "mssql" => DialectKind::SqlServer,
        "oracle" => DialectKind::Oracle,
        "h2" => DialectKind::H2,
        "clickhouse" => DialectKind::ClickHouse,
        "manticore" | "manticoresearch" => DialectKind::ManticoreSearch,
        "informix" => DialectKind::Informix,
        "questdb" => DialectKind::Questdb,
        _ => return Err(format!("Unknown dialect: {kind}. Supported: mysql, postgres, sqlite, duckdb, sqlserver, oracle, h2, clickhouse, manticore, informix, questdb")),
    };
    Ok(dialect_check(db_type))
}

#[tauri::command]
pub fn dialect_check_all_command() -> Vec<DialectInfo> {
    dialect_check_all()
}
