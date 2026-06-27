mod capabilities;
pub mod descriptor;
mod identifiers;
pub mod inference;
mod table_select;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod descriptor_snapshots;

pub use capabilities::{
    is_schema_aware, pagination_strategy, table_pagination_strategy, uses_fetch_first, PaginationContext,
    TablePaginationStrategy,
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
