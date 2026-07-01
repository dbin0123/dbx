use dbx_core::config::ConfigTree;
use dbx_core::config::{
    compute_config_checksum, detect_drift, ApprovalRecord, AuditQuery, AuditSummary, ConfigApproval, ConfigAuditEntry,
    ConfigAuditor, ConfigVersionSnapshot, DriftAlert, DriftDetector, DriftReport, TraceEntry, TraceRingBuffer,
};
use dbx_core::storage::Storage;
use std::sync::Arc;
use tauri::State;

// ---- Existing trace commands ----

#[tauri::command]
pub fn trace_export_command(entries: Vec<TraceEntry>, capacity: Option<usize>) -> Result<String, String> {
    let cap = capacity.unwrap_or(1000);
    let mut buf = TraceRingBuffer::new(cap);
    for entry in entries {
        buf.push(entry);
    }
    buf.export_json()
}

#[tauri::command]
pub fn trace_stats_command(entries: Vec<TraceEntry>) -> dbx_core::config::TraceStats {
    let mut buf = TraceRingBuffer::new(entries.len().max(1));
    for entry in entries {
        buf.push(entry);
    }
    buf.stats()
}

// ---- 16.1 Audit Commands ----

#[tauri::command]
pub async fn config_audit_record_command(
    storage: State<'_, Arc<Storage>>,
    operator: String,
    reason: String,
    key_path: String,
    change_diff: serde_json::Value,
    config_snapshot: serde_json::Value,
) -> Result<ConfigAuditEntry, String> {
    let auditor = ConfigAuditor::new((*storage).clone());
    auditor.record_change(&operator, &reason, &key_path, change_diff, config_snapshot).await
}

#[tauri::command]
pub async fn config_audit_query_command(
    storage: State<'_, Arc<Storage>>,
    key_path: Option<String>,
    operator: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<AuditSummary, String> {
    let auditor = ConfigAuditor::new((*storage).clone());
    let query = AuditQuery { key_path, operator, limit, offset };
    auditor.query_history(&query).await
}

#[tauri::command]
pub async fn config_snapshot_save_command(
    storage: State<'_, Arc<Storage>>,
    key_path: String,
    tree_json: serde_json::Value,
) -> Result<ConfigVersionSnapshot, String> {
    let auditor = ConfigAuditor::new((*storage).clone());
    let tree: ConfigTree = serde_json::from_value(tree_json).map_err(|e| format!("parse ConfigTree: {e}"))?;
    auditor.save_snapshot(&key_path, &tree).await
}

#[tauri::command]
pub async fn config_snapshot_list_command(
    storage: State<'_, Arc<Storage>>,
    key_path: String,
) -> Result<Vec<ConfigVersionSnapshot>, String> {
    let auditor = ConfigAuditor::new((*storage).clone());
    auditor.list_versions(&key_path).await
}

#[tauri::command]
pub async fn config_rollback_command(
    storage: State<'_, Arc<Storage>>,
    key_path: String,
    version: u64,
    operator: String,
    reason: String,
) -> Result<serde_json::Value, String> {
    let auditor = ConfigAuditor::new((*storage).clone());
    let tree = auditor.rollback(&key_path, version, &operator, &reason).await?;
    serde_json::to_value(&tree).map_err(|e| format!("serialize tree: {e}"))
}

// ---- 16.2 Approval Commands ----

#[tauri::command]
pub async fn config_approval_submit_command(
    storage: State<'_, Arc<Storage>>,
    domain: String,
    description: String,
    requester: String,
    draft_config: serde_json::Value,
    webhook_url: Option<String>,
) -> Result<ApprovalRecord, String> {
    let approval = ConfigApproval::new((*storage).clone());
    approval.submit_change(&domain, &description, &requester, draft_config, webhook_url.as_deref()).await
}

#[tauri::command]
pub async fn config_approval_approve_command(
    storage: State<'_, Arc<Storage>>,
    id: String,
    reviewer: String,
) -> Result<ApprovalRecord, String> {
    let approval = ConfigApproval::new((*storage).clone());
    approval.approve(&id, &reviewer).await
}

#[tauri::command]
pub async fn config_approval_reject_command(
    storage: State<'_, Arc<Storage>>,
    id: String,
    reviewer: String,
) -> Result<ApprovalRecord, String> {
    let approval = ConfigApproval::new((*storage).clone());
    approval.reject(&id, &reviewer).await
}

#[tauri::command]
pub async fn config_approval_list_pending_command(
    storage: State<'_, Arc<Storage>>,
) -> Result<Vec<ApprovalRecord>, String> {
    let approval = ConfigApproval::new((*storage).clone());
    approval.list_pending().await
}

#[tauri::command]
pub async fn config_approval_check_effective_command(
    storage: State<'_, Arc<Storage>>,
    domain: String,
) -> Result<bool, String> {
    let approval = ConfigApproval::new((*storage).clone());
    approval.check_effective(&domain).await
}

// ---- 16.3 Drift Detection Commands ----

#[tauri::command]
pub fn config_checksum_command(tree_json: serde_json::Value) -> Result<String, String> {
    let tree: ConfigTree = serde_json::from_value(tree_json).map_err(|e| format!("parse ConfigTree: {e}"))?;
    Ok(compute_config_checksum(&tree))
}

#[tauri::command]
pub fn config_detect_drift_command(
    source_tree_json: serde_json::Value,
    target_tree_json: serde_json::Value,
    key_path: String,
) -> Result<Option<DriftReport>, String> {
    let source: ConfigTree =
        serde_json::from_value(source_tree_json).map_err(|e| format!("parse source ConfigTree: {e}"))?;
    let target: ConfigTree =
        serde_json::from_value(target_tree_json).map_err(|e| format!("parse target ConfigTree: {e}"))?;
    Ok(detect_drift(&source, &target, &key_path))
}

#[tauri::command]
pub async fn config_drift_alert_record_command(
    storage: State<'_, Arc<Storage>>,
    alert: DriftAlert,
) -> Result<(), String> {
    let detector = DriftDetector::new((*storage).clone());
    detector.record_alert(&alert).await
}

#[tauri::command]
pub async fn config_drift_alert_acknowledge_command(
    storage: State<'_, Arc<Storage>>,
    id: String,
) -> Result<(), String> {
    let detector = DriftDetector::new((*storage).clone());
    detector.acknowledge_alert(&id).await
}

#[tauri::command]
pub async fn config_drift_alert_list_command(
    storage: State<'_, Arc<Storage>>,
    acknowledged: Option<bool>,
) -> Result<Vec<DriftAlert>, String> {
    let detector = DriftDetector::new((*storage).clone());
    detector.list_alerts(acknowledged).await
}
