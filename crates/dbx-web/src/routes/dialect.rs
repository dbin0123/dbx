use axum::extract::Query;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DialectDataTypesQuery {
    dialect_name: String,
}

pub async fn list_data_types(Query(query): Query<DialectDataTypesQuery>) -> Json<Vec<String>> {
    Json(dbx_core::sql_dialect::dialect_types::list_dialect_type_names(&query.dialect_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_data_types_matches_tauri_command_behavior() {
        let registry = dbx_core::sql_dialect::dialect_loader::DialectRegistry::global();
        eprintln!("registry len before init: {}", registry.len());
        dbx_core::sql_dialect::dialect_loader::register_core_dialects();
        eprintln!("registry len after init: {}", registry.len());

        eprintln!("Checking specific dialect keys:");
        for name in &["postgresql", "PostgreSQL", "mysql", "MySQL", "sqlite", "SQLite"] {
            eprintln!("  {}: {}", name, registry.get(name).is_some());
        }

        let Json(types) =
            list_data_types(Query(DialectDataTypesQuery { dialect_name: "PostgreSQL".to_string() })).await;
        eprintln!("types count: {}", types.len());

        assert!(
            types.is_empty() == false,
            "Expected non-empty types for PostgreSQL. Registry has entries but key mismatch."
        );
    }
}
