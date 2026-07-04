mod capabilities;
pub mod descriptor;
pub mod dialect_loader;
pub mod dialect_types;
pub mod dialect_yaml;
pub mod hot_reload;
mod identifiers;
pub mod inference;
mod table_select;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod descriptor_snapshots;

pub use capabilities::{
    firebird_rows_clause, is_schema_aware, pagination_strategy, table_pagination_strategy, uses_fetch_first,
    uses_single_row_insert_statements, PaginationContext, TablePaginationStrategy,
};
pub use descriptor::{
    dialect_check, dialect_check_all, DialectCapabilityDescriptor, DialectInfo, DialectKind, TypeConversionRule,
    TypeMappingMatrix, CAP_ADD_COLUMN, CAP_ALTER_EXISTING_COLUMN, CAP_ALTER_OWNER, CAP_ALTER_PRIMARY_KEY,
    CAP_AUTO_INCREMENT, CAP_COMMENT, CAP_CREATE_FUNCTION, CAP_CREATE_INDEX, CAP_CREATE_OR_REPLACE, CAP_CREATE_SEQUENCE,
    CAP_CREATE_TABLE, CAP_CREATE_TRIGGER, CAP_DROP_COLUMN, CAP_DROP_FUNCTION, CAP_DROP_INDEX, CAP_DROP_SEQUENCE,
    CAP_DROP_TABLE, CAP_DROP_TRIGGER, CAP_FOREIGN_KEY, CAP_GRANT_REVOKE, CAP_IDENTITY_COLUMNS, CAP_IF_NOT_EXISTS,
    CAP_INDEX_COMMENT, CAP_INDEX_FILTER, CAP_INDEX_INCLUDE, CAP_INDEX_TYPE, CAP_REBUILD_INDEX, CAP_RENAME_COLUMN,
    CAP_REORDER_COLUMN, CAP_TEMPORARY_TABLE, CAP_TRANSACTIONAL_DDL, CAP_TRUNCATE_TABLE,
};
pub use identifiers::{normalize_where_input, qualified_table_name, quote_table_identifier};
pub(crate) use identifiers::{parse_sqlserver_linked_schema_ref, qualified_transfer_table, quote_transfer_identifier};
pub use table_select::{build_count_table_sql, build_table_data_select_sql, build_table_select_sql};
pub use types::*;

// ============================================================================
// Global dialect resolution: YAML registry → hardcoded fallback
// ============================================================================

/// Resolve a dialect descriptor for the given kind.
/// Checks the YAML DialectRegistry first; falls back to hardcoded for_dialect().
pub fn resolve(kind: descriptor::DialectKind) -> descriptor::DialectCapabilityDescriptor {
    lazy_init();
    if let Some(desc) = dialect_loader::DialectRegistry::global().get_descriptor(kind.label()) {
        return desc;
    }
    descriptor::DialectCapabilityDescriptor::for_dialect(kind)
}

/// Ensure DML rules are loaded once.
fn lazy_init() {
    use std::sync::OnceLock;
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        let _ = crate::dml_binding::DmlCleanRuleRegistry::load_default();
    });
}

/// Convenience: resolve descriptor from database type (most common entry point).
pub fn resolve_for_db(db_type: crate::models::connection::DatabaseType) -> descriptor::DialectCapabilityDescriptor {
    let kind = descriptor::DialectKind::from_database_type(db_type);
    resolve(kind)
}

/// Resolve kind label (prefer YAML display_name, fallback to DialectKind::label).
pub fn resolve_label(kind: descriptor::DialectKind) -> String {
    if let Some(loaded) = dialect_loader::DialectRegistry::global().get(kind.label()) {
        return loaded.yaml.dialect.display_name.clone().unwrap_or_else(|| kind.label().to_string());
    }
    kind.label().to_string()
}
