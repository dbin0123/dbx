use std::collections::{HashMap, HashSet, VecDeque};

use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::models::connection::DatabaseType;
use crate::sql_dialect::descriptor::DialectKind;
use crate::sql_dialect::inference::{ColumnType, DefaultTypeInferenceEngine, TypeInferenceEngine};
use crate::types::{
    ColumnInfo, ForeignKeyInfo, FunctionInfo, IndexInfo, OwnerInfo, RuleInfo, SequenceInfo, TableInfo, TriggerInfo,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ColumnInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ColumnInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<IndexInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<IndexInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKeyDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ForeignKeyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ForeignKeyInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<TriggerInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TriggerInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FunctionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<FunctionInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SequenceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SequenceInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RuleInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<RuleInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub object_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<OwnerInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<OwnerInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<ColumnDiff>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexes: Option<Vec<IndexDiff>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_keys: Option<Vec<ForeignKeyDiff>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<TriggerDiff>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ddl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ddl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_table_comment: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_table_comment: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_sql: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchemaDetail {
    pub name: String,
    #[serde(default)]
    pub columns: Vec<ColumnInfo>,
    #[serde(default)]
    pub indexes: Vec<IndexInfo>,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKeyInfo>,
    #[serde(default)]
    pub triggers: Vec<TriggerInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ddl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDiffPreparationOptions {
    #[serde(default)]
    pub source_tables: Vec<TableInfo>,
    #[serde(default)]
    pub target_tables: Vec<TableInfo>,
    #[serde(default)]
    pub source_details: Vec<TableSchemaDetail>,
    #[serde(default)]
    pub target_details: Vec<TableSchemaDetail>,
    #[serde(default)]
    pub source_functions: Vec<FunctionInfo>,
    #[serde(default)]
    pub target_functions: Vec<FunctionInfo>,
    #[serde(default)]
    pub source_sequences: Vec<SequenceInfo>,
    #[serde(default)]
    pub target_sequences: Vec<SequenceInfo>,
    #[serde(default)]
    pub source_rules: Vec<RuleInfo>,
    #[serde(default)]
    pub target_rules: Vec<RuleInfo>,
    #[serde(default)]
    pub source_owners: Vec<OwnerInfo>,
    #[serde(default)]
    pub target_owners: Vec<OwnerInfo>,
    pub database_type: DatabaseType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_schema: Option<String>,
    #[serde(default)]
    pub ignore_comments: bool,
    #[serde(default)]
    pub cascade_delete: bool,
    #[serde(default)]
    pub compare_column_order: bool,
    #[serde(default)]
    pub detect_renames: bool,
    #[serde(default)]
    pub rename_threshold: f64,
    #[serde(default)]
    pub enable_rollback: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub batch_patterns: Vec<BatchPattern>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dialect: Option<DialectKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_dialect: Option<DialectKind>,
    #[serde(default)]
    pub compatibility_threshold: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_permissions: Vec<PermissionInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_permissions: Vec<PermissionInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shard_strategy: Option<ShardStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_constraint: Option<ResourceConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDiffPreparation {
    pub diffs: Vec<TableDiff>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub function_diffs: Vec<FunctionDiff>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequence_diffs: Vec<SequenceDiff>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_diffs: Vec<RuleDiff>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_diffs: Vec<OwnerDiff>,
    pub sync_sql: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_sync_sql: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rename_candidates: Vec<RenameCandidate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_graph: Option<RollbackGraph>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compatibility_warnings: Vec<ColumnCompatibilityWarning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permission_diffs: Vec<PermissionDiff>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permission_sync_sql: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_graph: Option<DependencyGraph>,
}

// ============================================================================
// Phase 4.1: Dependency Graph & Rename Detection
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub table_name: String,
    pub depends_on: Vec<String>,
    pub depended_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub topological_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub level1_score: f64,
    pub level2_score: f64,
    pub composite_score: f64,
    pub level1_covered: u64,
    pub level1_total: u64,
    pub level2_covered: u64,
    pub level2_total: u64,
    pub uncovered_edges: Vec<UncoveredEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredEdge {
    pub from_table: String,
    pub to_table: String,
    pub level: u32,
}

impl DependencyGraph {
    pub fn build(details: &[TableSchemaDetail], tables: &[TableInfo]) -> Self {
        let table_names: HashSet<&str> =
            tables.iter().filter(|t| !t.table_type.contains("VIEW")).map(|t| t.name.as_str()).collect();
        let mut nodes: HashMap<String, DependencyNode> = table_names
            .iter()
            .map(|name| {
                (
                    name.to_string(),
                    DependencyNode { table_name: name.to_string(), depends_on: Vec::new(), depended_by: Vec::new() },
                )
            })
            .collect();
        let detail_map: HashMap<&str, &TableSchemaDetail> = details.iter().map(|d| (d.name.as_str(), d)).collect();

        for table_name in &table_names {
            if let Some(detail) = detail_map.get(table_name) {
                for fk in &detail.foreign_keys {
                    if table_names.contains(fk.ref_table.as_str()) {
                        if let Some(node) = nodes.get_mut(*table_name) {
                            if !node.depends_on.contains(&fk.ref_table) {
                                node.depends_on.push(fk.ref_table.clone());
                            }
                        }
                        if let Some(ref_node) = nodes.get_mut(&fk.ref_table) {
                            if !ref_node.depended_by.iter().any(|d| d == *table_name) {
                                ref_node.depended_by.push((*table_name).to_string());
                            }
                        }
                    }
                }
            }
        }

        let topological_order = Self::topological_sort(&nodes);
        DependencyGraph { nodes, topological_order }
    }

    fn topological_sort(nodes: &HashMap<String, DependencyNode>) -> Vec<String> {
        let mut in_degree: HashMap<&str, usize> = nodes.keys().map(|k| (k.as_str(), 0usize)).collect();
        for node in nodes.values() {
            in_degree.entry(node.table_name.as_str()).or_insert(0);
            for _dep in &node.depends_on {
                *in_degree.entry(node.table_name.as_str()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<&str> = in_degree.iter().filter(|(_, &deg)| deg == 0).map(|(&name, _)| name).collect();

        let mut result = Vec::new();
        while let Some(name) = queue.pop_front() {
            result.push(name.to_string());
            if let Some(node) = nodes.get(name) {
                for dependent in &node.depended_by {
                    if let Some(deg) = in_degree.get_mut(dependent.as_str()) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.as_str());
                        }
                    }
                }
            }
        }

        if result.len() != nodes.len() {
            let remaining: Vec<String> = nodes.keys().filter(|k| !result.contains(k)).cloned().collect();
            result.extend(remaining);
        }

        result
    }

    pub fn build_order(&self) -> Vec<String> {
        self.topological_order.clone()
    }

    pub fn drop_order(&self) -> Vec<String> {
        let mut order = self.topological_order.clone();
        order.reverse();
        order
    }

    pub fn coverage_score(&self, diffed_tables: &[String]) -> f64 {
        self.coverage_score_level1(diffed_tables)
    }

    pub fn coverage_score_level1(&self, diffed_tables: &[String]) -> f64 {
        if self.nodes.is_empty() {
            return 1.0;
        }
        let diffed_set: HashSet<&str> = diffed_tables.iter().map(|s| s.as_str()).collect();
        let mut covered_edges = 0u64;
        let mut total_edges = 0u64;

        for node in self.nodes.values() {
            for dep in &node.depends_on {
                total_edges += 1;
                if diffed_set.contains(node.table_name.as_str()) && diffed_set.contains(dep.as_str()) {
                    covered_edges += 1;
                }
            }
        }

        if total_edges == 0 {
            1.0
        } else {
            covered_edges as f64 / total_edges as f64
        }
    }

    pub fn coverage_score_level2(&self, diffed_tables: &[String]) -> f64 {
        if self.nodes.is_empty() {
            return 1.0;
        }
        let diffed_set: HashSet<&str> = diffed_tables.iter().map(|s| s.as_str()).collect();

        let mut transitive_edges = 0u64;
        let mut covered_transitive = 0u64;

        for node in self.nodes.values() {
            let table_name = node.table_name.as_str();
            if !diffed_set.contains(table_name) {
                continue;
            }
            for indirect in &node.depends_on {
                if let Some(inner) = self.nodes.get(indirect) {
                    for grand in &inner.depends_on {
                        transitive_edges += 1;
                        if diffed_set.contains(table_name) && diffed_set.contains(grand.as_str()) {
                            covered_transitive += 1;
                        }
                    }
                }
            }
        }

        if transitive_edges == 0 {
            1.0
        } else {
            covered_transitive as f64 / transitive_edges as f64
        }
    }

    pub fn composite_coverage_score(&self, diffed_tables: &[String]) -> CoverageReport {
        let diffed_set: HashSet<&str> = diffed_tables.iter().map(|s| s.as_str()).collect();

        let (l1_covered, l1_total) = self.count_edges(diffed_tables, &diffed_set, 1);
        let (l2_covered, l2_total) = self.count_transitive_edges(diffed_tables, &diffed_set);

        let l1_score = if l1_total == 0 { 1.0 } else { l1_covered as f64 / l1_total as f64 };
        let l2_score = if l2_total == 0 { 1.0 } else { l2_covered as f64 / l2_total as f64 };

        let composite_score = 0.6 * l1_score + 0.4 * l2_score;

        let uncovered = self.collect_uncovered_edges(diffed_tables, &diffed_set);

        CoverageReport {
            level1_score: l1_score,
            level2_score: l2_score,
            composite_score,
            level1_covered: l1_covered,
            level1_total: l1_total,
            level2_covered: l2_covered,
            level2_total: l2_total,
            uncovered_edges: uncovered,
        }
    }

    fn count_edges(&self, _diffed_tables: &[String], diffed_set: &HashSet<&str>, _level: u32) -> (u64, u64) {
        let mut covered = 0u64;
        let mut total = 0u64;
        for node in self.nodes.values() {
            for dep in &node.depends_on {
                total += 1;
                if diffed_set.contains(node.table_name.as_str()) && diffed_set.contains(dep.as_str()) {
                    covered += 1;
                }
            }
        }
        (covered, total)
    }

    fn count_transitive_edges(&self, _diffed_tables: &[String], diffed_set: &HashSet<&str>) -> (u64, u64) {
        let mut covered = 0u64;
        let mut total = 0u64;
        for node in self.nodes.values() {
            let table_name = node.table_name.as_str();
            if !diffed_set.contains(table_name) {
                continue;
            }
            for indirect in &node.depends_on {
                if let Some(inner) = self.nodes.get(indirect) {
                    for grand in &inner.depends_on {
                        total += 1;
                        if diffed_set.contains(table_name) && diffed_set.contains(grand.as_str()) {
                            covered += 1;
                        }
                    }
                }
            }
        }
        (covered, total)
    }

    fn collect_uncovered_edges(&self, _diffed_tables: &[String], diffed_set: &HashSet<&str>) -> Vec<UncoveredEdge> {
        let mut uncovered = Vec::new();
        for node in self.nodes.values() {
            for dep in &node.depends_on {
                let both_covered = diffed_set.contains(node.table_name.as_str()) && diffed_set.contains(dep.as_str());
                if !both_covered {
                    uncovered.push(UncoveredEdge {
                        from_table: node.table_name.clone(),
                        to_table: dep.clone(),
                        level: 1,
                    });
                }
            }
            for indirect in &node.depends_on {
                if let Some(inner) = self.nodes.get(indirect) {
                    for grand in &inner.depends_on {
                        let all_covered =
                            diffed_set.contains(node.table_name.as_str()) && diffed_set.contains(grand.as_str());
                        if !all_covered {
                            uncovered.push(UncoveredEdge {
                                from_table: node.table_name.clone(),
                                to_table: grand.clone(),
                                level: 2,
                            });
                        }
                    }
                }
            }
        }
        uncovered
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameCandidate {
    pub removed_name: String,
    pub added_name: String,
    pub score: f64,
    pub column_jaccard: f64,
    pub type_similarity: f64,
}

fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn column_type_similarity(source_cols: &[ColumnInfo], target_cols: &[ColumnInfo]) -> f64 {
    if source_cols.is_empty() || target_cols.is_empty() {
        return 0.0;
    }
    let engine = DefaultTypeInferenceEngine;
    let source_map: HashMap<&str, &ColumnInfo> = source_cols.iter().map(|c| (c.name.as_str(), c)).collect();
    let target_map: HashMap<&str, &ColumnInfo> = target_cols.iter().map(|c| (c.name.as_str(), c)).collect();
    let common_names: HashSet<&str> = source_map.keys().filter(|k| target_map.contains_key(**k)).copied().collect();

    if common_names.is_empty() {
        return 0.0;
    }

    let total: f64 = common_names
        .iter()
        .map(|name| {
            let s = ColumnType::parse(&source_map[name].data_type);
            let t = ColumnType::parse(&target_map[name].data_type);
            engine.type_compatibility_score(&s, &t)
        })
        .sum();
    total / common_names.len() as f64
}

pub fn detect_renames(
    removed: &[String],
    added: &[String],
    source_details: &[TableSchemaDetail],
    target_details: &[TableSchemaDetail],
    threshold: f64,
) -> Vec<RenameCandidate> {
    let source_detail_map: HashMap<&str, &TableSchemaDetail> =
        source_details.iter().map(|d| (d.name.as_str(), d)).collect();
    let target_detail_map: HashMap<&str, &TableSchemaDetail> =
        target_details.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut candidates = Vec::new();
    for removed_name in removed {
        let Some(removed_detail) = target_detail_map.get(removed_name.as_str()) else { continue };
        for added_name in added {
            let Some(added_detail) = source_detail_map.get(added_name.as_str()) else { continue };

            let col_names_added: HashSet<String> = added_detail.columns.iter().map(|c| c.name.clone()).collect();
            let col_names_removed: HashSet<String> = removed_detail.columns.iter().map(|c| c.name.clone()).collect();
            let column_jaccard = jaccard_similarity(&col_names_removed, &col_names_added);

            let type_sim = column_type_similarity(&removed_detail.columns, &added_detail.columns);

            let score = column_jaccard * 0.6 + type_sim * 0.4;

            if score >= threshold {
                candidates.push(RenameCandidate {
                    removed_name: removed_name.clone(),
                    added_name: added_name.clone(),
                    score,
                    column_jaccard,
                    type_similarity: type_sim,
                });
            }
        }
    }

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut final_candidates = Vec::new();
    let mut used_removed: HashSet<String> = HashSet::new();
    let mut used_added: HashSet<String> = HashSet::new();

    for c in &candidates {
        if !used_removed.contains(&c.removed_name) && !used_added.contains(&c.added_name) {
            final_candidates.push(c.clone());
            used_removed.insert(c.removed_name.clone());
            used_added.insert(c.added_name.clone());
        }
    }

    final_candidates
}

// ============================================================================
// Phase 4.2: Batch Naming Pattern Recognition
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPattern {
    pub pattern: String,
    pub is_regex: bool,
    pub description: String,
}

pub fn diff_names_with_patterns(
    source: &[String],
    target: &[String],
    patterns: &[BatchPattern],
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<Vec<String>>) {
    let (added, removed, common) = diff_names(source, target);

    let mut pattern_matches: Vec<Vec<String>> = Vec::new();
    for pattern in patterns {
        let mut matches = Vec::new();
        if pattern.is_regex {
            if let Ok(re) = Regex::new(&pattern.pattern) {
                for name in source {
                    if re.is_match(name) {
                        matches.push(name.clone());
                    }
                }
            }
        } else {
            let glob_pattern = pattern.pattern.replace('*', ".*").replace('?', ".");
            if let Ok(re) = Regex::new(&format!("^{}$", glob_pattern)) {
                for name in source {
                    if re.is_match(name) {
                        matches.push(name.clone());
                    }
                }
            }
        }
        if !matches.is_empty() {
            pattern_matches.push(matches);
        }
    }

    (added, removed, common, pattern_matches)
}

pub fn detect_pattern_conflicts(patterns: &[BatchPattern], names: &[String]) -> Vec<Vec<String>> {
    let mut conflicts = Vec::new();
    for i in 0..patterns.len() {
        for j in (i + 1)..patterns.len() {
            let pi = &patterns[i];
            let pj = &patterns[j];
            let pattern_i =
                if pi.is_regex { pi.pattern.clone() } else { pi.pattern.replace('*', ".*").replace('?', ".") };
            let pattern_j =
                if pj.is_regex { pj.pattern.clone() } else { pj.pattern.replace('*', ".*").replace('?', ".") };

            let re_i = Regex::new(&format!("^{}$", pattern_i));
            let re_j = Regex::new(&format!("^{}$", pattern_j));
            if let (Ok(ri), Ok(rj)) = (re_i, re_j) {
                for name in names {
                    if ri.is_match(name) && rj.is_match(name) {
                        conflicts.push(vec![pi.description.clone(), pj.description.clone()]);
                        break;
                    }
                }
            }
        }
    }
    conflicts
}

// ============================================================================
// Phase 4.3: Dialect-Aware Type Compatibility Scoring
// ============================================================================

pub fn diff_columns_with_compatibility(
    source: &[ColumnInfo],
    target: &[ColumnInfo],
    ignore_comments: bool,
    compare_column_order: bool,
    source_dialect: DialectKind,
    target_dialect: DialectKind,
    compatibility_threshold: f64,
) -> (Vec<ColumnDiff>, Vec<ColumnCompatibilityWarning>) {
    use crate::sql_dialect::descriptor::TypeMappingMatrix;

    let matrix = TypeMappingMatrix::for_dialects(source_dialect, target_dialect);
    let engine = DefaultTypeInferenceEngine;

    let basic_diffs = diff_columns_with_options(source, target, ignore_comments, compare_column_order, false, 0.5);

    let mut warnings = Vec::new();
    let mut enhanced_diffs = Vec::new();

    for diff in basic_diffs {
        let mut warning = None;

        if diff.diff_type == "modified" {
            if let (Some(src), Some(tgt)) = (&diff.source, &diff.target) {
                let src_parsed = ColumnType::parse(&src.data_type);
                let tgt_parsed = ColumnType::parse(&tgt.data_type);
                let compatibility = engine.type_compatibility_score(&src_parsed, &tgt_parsed);

                let (mapped_type, requires_cast) = matrix.convert_type(&tgt.data_type);

                let risk = if compatibility >= 0.9 {
                    ColumnConversionRisk::None
                } else if compatibility >= 0.7 {
                    ColumnConversionRisk::Low
                } else if compatibility >= 0.5 {
                    ColumnConversionRisk::Medium
                } else {
                    ColumnConversionRisk::High
                };

                if compatibility < compatibility_threshold {
                    warning = Some(ColumnCompatibilityWarning {
                        column_name: diff.name.clone(),
                        source_type: src.data_type.clone(),
                        target_type: tgt.data_type.clone(),
                        compatibility_score: compatibility,
                        suggested_mapping: mapped_type,
                        requires_cast,
                        risk,
                    });
                }
            }
        }

        enhanced_diffs.push(diff);
        if let Some(w) = warning {
            warnings.push(w);
        }
    }

    (enhanced_diffs, warnings)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnCompatibilityWarning {
    pub column_name: String,
    pub source_type: String,
    pub target_type: String,
    pub compatibility_score: f64,
    pub suggested_mapping: String,
    pub requires_cast: bool,
    pub risk: ColumnConversionRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ColumnConversionRisk {
    None,
    Low,
    Medium,
    High,
}

// ============================================================================
// Phase 4.4: Bidirectional Diff & Rollback Graph
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffNode {
    pub table_diff: TableDiff,
    pub direction: DiffDirection,
    pub dependency_order: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiffDirection {
    Forward,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackGraph {
    pub forward_nodes: Vec<DiffNode>,
    pub rollback_nodes: Vec<DiffNode>,
    pub is_consistent: bool,
    pub consistency_issues: Vec<String>,
}

impl RollbackGraph {
    pub fn from_forward_diffs(
        forward_diffs: &[TableDiff],
        renames: &[RenameCandidate],
        dep_graph: &DependencyGraph,
    ) -> Self {
        let mut forward_nodes = Vec::new();
        let mut rollback_nodes = Vec::new();
        let consistency_issues = Vec::new();

        let rename_map: HashMap<&str, &RenameCandidate> =
            renames.iter().map(|r| (r.removed_name.as_str(), r)).collect();
        let rename_reverse: HashMap<&str, &str> =
            renames.iter().map(|r| (r.added_name.as_str(), r.removed_name.as_str())).collect();

        let order_map: HashMap<&str, usize> =
            dep_graph.topological_order.iter().enumerate().map(|(i, name)| (name.as_str(), i)).collect();

        for diff in forward_diffs {
            let order = order_map.get(diff.name.as_str()).copied().unwrap_or(usize::MAX);

            let (rename_source, rename_target, rename_score) = if diff.diff_type == "added" {
                if let Some(rc) = rename_reverse.get(diff.name.as_str()) {
                    (Some(rc.to_string()), Some(diff.name.clone()), None)
                } else {
                    (None, None, None)
                }
            } else if diff.diff_type == "removed" {
                if let Some(rc) = rename_map.get(diff.name.as_str()) {
                    (Some(diff.name.clone()), Some(rc.added_name.clone()), Some(rc.score))
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            };

            forward_nodes.push(DiffNode {
                table_diff: diff.clone(),
                direction: DiffDirection::Forward,
                dependency_order: order,
                rename_source,
                rename_target,
                rename_score,
            });

            let rollback_diff = Self::invert_diff(diff);
            rollback_nodes.push(DiffNode {
                table_diff: rollback_diff,
                direction: DiffDirection::Rollback,
                dependency_order: order,
                rename_source: None,
                rename_target: None,
                rename_score: None,
            });
        }

        RollbackGraph { forward_nodes, rollback_nodes, is_consistent: false, consistency_issues }
    }

    fn invert_diff_type(dt: &str) -> &str {
        match dt {
            "added" => "removed",
            "removed" => "added",
            "renamed" => "renamed",
            _ => "modified",
        }
    }

    fn invert_change_string(ch: &str) -> String {
        if let Some(_pos) = ch.find(" → ") {
            let parts: Vec<&str> = ch.split(" → ").collect();
            if parts.len() == 2 {
                format!("{} → {}", parts[1], parts[0])
            } else {
                ch.to_string()
            }
        } else {
            ch.to_string()
        }
    }

    fn invert_columns(cols: &[ColumnDiff]) -> Vec<ColumnDiff> {
        cols.iter()
            .map(|c| {
                let inverted_name = if c.diff_type == "renamed" {
                    c.target.as_ref().map(|t| t.name.clone()).unwrap_or_else(|| c.name.clone())
                } else {
                    c.name.clone()
                };
                ColumnDiff {
                    diff_type: Self::invert_diff_type(&c.diff_type).to_string(),
                    name: inverted_name,
                    source: c.target.clone(),
                    target: c.source.clone(),
                    changes: c.changes.iter().map(|ch| Self::invert_change_string(ch)).collect(),
                }
            })
            .collect()
    }

    fn invert_indexes(idxs: &[IndexDiff]) -> Vec<IndexDiff> {
        idxs.iter()
            .map(|i| IndexDiff {
                diff_type: Self::invert_diff_type(&i.diff_type).to_string(),
                name: i.name.clone(),
                source: i.target.clone(),
                target: i.source.clone(),
                changes: i.changes.clone(),
            })
            .collect()
    }

    fn invert_fks(fks: &[ForeignKeyDiff]) -> Vec<ForeignKeyDiff> {
        fks.iter()
            .map(|fk| ForeignKeyDiff {
                diff_type: Self::invert_diff_type(&fk.diff_type).to_string(),
                name: fk.name.clone(),
                source: fk.target.clone(),
                target: fk.source.clone(),
                changes: fk.changes.clone(),
            })
            .collect()
    }

    fn invert_triggers(trgs: &[TriggerDiff]) -> Vec<TriggerDiff> {
        trgs.iter()
            .map(|t| TriggerDiff {
                diff_type: Self::invert_diff_type(&t.diff_type).to_string(),
                name: t.name.clone(),
                source: t.target.clone(),
                target: t.source.clone(),
                changes: t.changes.clone(),
            })
            .collect()
    }

    fn invert_diff(diff: &TableDiff) -> TableDiff {
        let inverted_type = Self::invert_diff_type(&diff.diff_type).to_string();

        let inverted_columns = diff.columns.as_ref().map(|cols| Self::invert_columns(cols));
        let inverted_indexes = diff.indexes.as_ref().map(|idxs| Self::invert_indexes(idxs));
        let inverted_fks = diff.foreign_keys.as_ref().map(|fks| Self::invert_fks(fks));
        let inverted_triggers = diff.triggers.as_ref().map(|trgs| Self::invert_triggers(trgs));

        let (source_comment, target_comment) = match inverted_type.as_str() {
            "added" => (diff.target_table_comment.clone(), diff.source_table_comment.clone()),
            "removed" => (diff.source_table_comment.clone(), diff.target_table_comment.clone()),
            _ => (diff.target_table_comment.clone(), diff.source_table_comment.clone()),
        };

        TableDiff {
            diff_type: inverted_type,
            object_type: diff.object_type.clone(),
            name: diff.name.clone(),
            columns: inverted_columns,
            indexes: inverted_indexes,
            foreign_keys: inverted_fks,
            triggers: inverted_triggers,
            ddl: diff.target_ddl.clone(),
            target_ddl: diff.ddl.clone(),
            source_table_comment: source_comment,
            target_table_comment: target_comment,
            sync_sql: None,
        }
    }

    pub fn validate_consistency(&mut self) -> bool {
        self.consistency_issues.clear();

        for fwd in &self.forward_nodes {
            let has_rollback = self.rollback_nodes.iter().any(|rbk| {
                rbk.table_diff.name == fwd.table_diff.name
                    && match (fwd.table_diff.diff_type.as_str(), rbk.table_diff.diff_type.as_str()) {
                        ("added", "removed") | ("removed", "added") => true,
                        ("modified", "modified") => true,
                        ("none", "none") => true,
                        _ => false,
                    }
            });

            if !has_rollback {
                self.consistency_issues.push(format!(
                    "No rollback entry for forward {}: {}",
                    fwd.table_diff.diff_type, fwd.table_diff.name
                ));
            }

            let rollback_of_rollback: Vec<_> = self
                .rollback_nodes
                .iter()
                .filter(|rbk| rbk.table_diff.name == fwd.table_diff.name)
                .map(|rbk| Self::invert_diff(&rbk.table_diff))
                .collect();

            for ror in &rollback_of_rollback {
                if ror.diff_type != fwd.table_diff.diff_type {
                    self.consistency_issues.push(format!(
                        "Forward∘Rollback mismatch for {}: forward={}, rollback∘rollback={}",
                        fwd.table_diff.name, fwd.table_diff.diff_type, ror.diff_type
                    ));
                }
            }
        }

        self.is_consistent = self.consistency_issues.is_empty();
        self.is_consistent
    }
}

pub fn generate_rollback_sync_sql(
    rollback_graph: &RollbackGraph,
    db_type: DatabaseType,
    schema: Option<&str>,
    cascade_delete: bool,
) -> String {
    let rollback_diffs: Vec<TableDiff> = rollback_graph.rollback_nodes.iter().map(|n| n.table_diff.clone()).collect();
    generate_schema_sync_sql(&rollback_diffs, &[], &[], &[], &[], db_type, schema, cascade_delete, None)
}

// ============================================================================
// Phase 4.5: Shard-Parallel Comparison
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStrategy {
    pub shard_count: usize,
    pub shard_by: ShardBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShardBy {
    Table,
    Schema,
    RoundRobin,
}

pub fn shard_diff(options: &SchemaDiffPreparationOptions, shard_strategy: &ShardStrategy) -> Vec<TableDiff> {
    let table_count = options.source_tables.len().max(options.target_tables.len());
    let shard_count = shard_strategy.shard_count.min(table_count.max(1));

    if shard_count <= 1 {
        return diff_schema(options);
    }

    let source_table_names: Vec<&str> =
        options.source_tables.iter().filter(|t| !t.table_type.contains("VIEW")).map(|t| t.name.as_str()).collect();
    let source_view_names: Vec<&str> =
        options.source_tables.iter().filter(|t| t.table_type.contains("VIEW")).map(|t| t.name.as_str()).collect();
    let _target_table_names: Vec<&str> =
        options.target_tables.iter().filter(|t| !t.table_type.contains("VIEW")).map(|t| t.name.as_str()).collect();
    let _target_view_names: Vec<&str> =
        options.target_tables.iter().filter(|t| t.table_type.contains("VIEW")).map(|t| t.name.as_str()).collect();

    let source_all: Vec<&str> = source_table_names.iter().chain(source_view_names.iter()).copied().collect();
    let shards: Vec<Vec<&str>> = match &shard_strategy.shard_by {
        ShardBy::Table | ShardBy::RoundRobin => {
            let mut s: Vec<Vec<&str>> = vec![Vec::new(); shard_count];
            for (i, name) in source_all.iter().enumerate() {
                s[i % shard_count].push(*name);
            }
            s
        }
        ShardBy::Schema => {
            let mut schema_groups: HashMap<&str, Vec<&str>> = HashMap::new();
            for table in &options.source_tables {
                let schema = table.parent_schema.as_deref().unwrap_or("default");
                schema_groups.entry(schema).or_default().push(table.name.as_str());
            }
            let mut s: Vec<Vec<&str>> = vec![Vec::new(); shard_count];
            for (i, (_schema, names)) in schema_groups.iter().enumerate() {
                s[i % shard_count].extend(names);
            }
            s
        }
    };

    let shard_results: Vec<Vec<TableDiff>> = shards
        .par_iter()
        .filter(|shard| !shard.is_empty())
        .map(|shard| {
            let shard_set: HashSet<&str> = shard.iter().copied().collect();
            let shard_options = SchemaDiffPreparationOptions {
                source_tables: options
                    .source_tables
                    .iter()
                    .filter(|t| shard_set.contains(t.name.as_str()))
                    .cloned()
                    .collect(),
                target_tables: options
                    .target_tables
                    .iter()
                    .filter(|t| shard_set.contains(t.name.as_str()))
                    .cloned()
                    .collect(),
                source_details: options
                    .source_details
                    .iter()
                    .filter(|d| shard_set.contains(d.name.as_str()))
                    .cloned()
                    .collect(),
                target_details: options
                    .target_details
                    .iter()
                    .filter(|d| shard_set.contains(d.name.as_str()))
                    .cloned()
                    .collect(),
                source_functions: options.source_functions.clone(),
                target_functions: options.target_functions.clone(),
                source_sequences: options.source_sequences.clone(),
                target_sequences: options.target_sequences.clone(),
                source_rules: options.source_rules.clone(),
                target_rules: options.target_rules.clone(),
                source_owners: options.source_owners.clone(),
                target_owners: options.target_owners.clone(),
                database_type: options.database_type,
                target_schema: options.target_schema.clone(),
                ignore_comments: options.ignore_comments,
                cascade_delete: options.cascade_delete,
                compare_column_order: options.compare_column_order,
                ..Default::default()
            };
            diff_schema(&shard_options)
        })
        .collect();

    let mut merged: Vec<TableDiff> = Vec::new();
    for shard_result in shard_results {
        merged.extend(shard_result);
    }

    merged.sort_by(|a, b| a.name.cmp(&b.name));
    merged.dedup_by(|a, b| a.name == b.name && a.diff_type == b.diff_type);
    merged
}

// ============================================================================
// Phase 4.6: Permission & Role-Aware Sync
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionInfo {
    pub grantee: String,
    pub object_type: String,
    pub object_name: String,
    pub privilege: String,
    pub is_grantable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDiff {
    #[serde(rename = "type")]
    pub diff_type: String,
    pub grantee: String,
    pub object_name: String,
    pub privilege: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PermissionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<PermissionInfo>,
}

pub fn diff_permissions(source: &[PermissionInfo], target: &[PermissionInfo]) -> Vec<PermissionDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<(&str, &str, &str), &PermissionInfo> =
        target.iter().map(|p| ((p.grantee.as_str(), p.object_name.as_str(), p.privilege.as_str()), p)).collect();
    let source_map: HashMap<(&str, &str, &str), &PermissionInfo> =
        source.iter().map(|p| ((p.grantee.as_str(), p.object_name.as_str(), p.privilege.as_str()), p)).collect();

    for sp in source {
        let key = (sp.grantee.as_str(), sp.object_name.as_str(), sp.privilege.as_str());
        if !target_map.contains_key(&key) {
            diffs.push(PermissionDiff {
                diff_type: "added".to_string(),
                grantee: sp.grantee.clone(),
                object_name: sp.object_name.clone(),
                privilege: sp.privilege.clone(),
                source: Some(sp.clone()),
                target: None,
            });
        }
    }

    for tp in target {
        let key = (tp.grantee.as_str(), tp.object_name.as_str(), tp.privilege.as_str());
        if !source_map.contains_key(&key) {
            diffs.push(PermissionDiff {
                diff_type: "removed".to_string(),
                grantee: tp.grantee.clone(),
                object_name: tp.object_name.clone(),
                privilege: tp.privilege.clone(),
                source: None,
                target: Some(tp.clone()),
            });
        }
    }

    diffs
}

pub fn generate_permission_sync_sql(diffs: &[PermissionDiff], db_type: DatabaseType, schema: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let is_mysql = matches!(
        db_type,
        DatabaseType::Mysql
            | DatabaseType::Doris
            | DatabaseType::StarRocks
            | DatabaseType::Goldendb
            | DatabaseType::Sundb
            | DatabaseType::Databend
            | DatabaseType::Gbase
    );

    for diff in diffs {
        match diff.diff_type.as_str() {
            "added" => {
                if let Some(source) = &diff.source {
                    if is_mysql {
                        let object_path = if let Some(sch) = schema {
                            format!("`{}`.`{}`", sch.replace('`', "``"), source.object_name.replace('`', "``"))
                        } else {
                            format!("`{}`", source.object_name.replace('`', "``"))
                        };
                        let with_grant = if source.is_grantable { " WITH GRANT OPTION" } else { "" };
                        let grantee_escaped = source.grantee.replace('\'', "''");
                        lines.push(format!(
                            "GRANT {} ON {} TO '{}'{};",
                            source.privilege, object_path, grantee_escaped, with_grant
                        ));
                    } else {
                        let obj_escaped = source.object_name.replace('"', "\"\"");
                        let object_path = if let Some(sch) = schema {
                            format!("{} \"{}\".\"{}\"", source.object_type, sch, obj_escaped)
                        } else {
                            format!("{} \"{}\"", source.object_type, obj_escaped)
                        };
                        let with_grant = if source.is_grantable { " WITH GRANT OPTION" } else { "" };
                        let grantee_escaped = source.grantee.replace('"', "\"\"");
                        lines.push(format!(
                            "GRANT {} ON {} TO \"{}\"{};",
                            source.privilege, object_path, grantee_escaped, with_grant
                        ));
                    }
                }
            }
            "removed" => {
                if let Some(target) = &diff.target {
                    if is_mysql {
                        let object_path = if let Some(sch) = schema {
                            format!("`{}`.`{}`", sch.replace('`', "``"), target.object_name.replace('`', "``"))
                        } else {
                            format!("`{}`", target.object_name.replace('`', "``"))
                        };
                        let grantee_escaped = target.grantee.replace('\'', "''");
                        lines.push(format!(
                            "REVOKE {} ON {} FROM '{}';",
                            target.privilege, object_path, grantee_escaped
                        ));
                    } else {
                        let obj_escaped = target.object_name.replace('"', "\"\"");
                        let object_path = if let Some(sch) = schema {
                            format!("{} \"{}\".\"{}\"", target.object_type, sch, obj_escaped)
                        } else {
                            format!("{} \"{}\"", target.object_type, obj_escaped)
                        };
                        let grantee_escaped = target.grantee.replace('"', "\"\"");
                        lines.push(format!(
                            "REVOKE {} ON {} FROM \"{}\";",
                            target.privilege, object_path, grantee_escaped
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    lines.join("\n")
}

// ============================================================================
// Phase 4.7: Metadata Resource-Aware Scheduling
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraint {
    pub max_concurrent_connections: usize,
    pub max_memory_mb: u64,
    pub max_tables_per_batch: usize,
    pub throttle_delay_ms: u64,
}

impl Default for ResourceConstraint {
    fn default() -> Self {
        Self { max_concurrent_connections: 4, max_memory_mb: 512, max_tables_per_batch: 50, throttle_delay_ms: 100 }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveScheduler {
    pub constraint: ResourceConstraint,
    pub current_connections: usize,
    pub estimated_table_count: usize,
}

impl AdaptiveScheduler {
    pub fn new(constraint: ResourceConstraint, table_count: usize) -> Self {
        Self { constraint, current_connections: 0, estimated_table_count: table_count }
    }

    pub fn optimal_batch_size(&self) -> usize {
        let conn_limit = self.constraint.max_concurrent_connections;
        let mem_limit = self.constraint.max_memory_mb as usize * 50;
        let table_limit = self.constraint.max_tables_per_batch;

        let batches = self.estimated_table_count.max(1);
        let per_batch = (self.estimated_table_count / conn_limit).max(1);

        per_batch.min(mem_limit / batches).min(table_limit)
    }

    pub fn recommended_shard_count(&self) -> usize {
        let per_batch = self.optimal_batch_size();
        let count = (self.estimated_table_count as f64 / per_batch as f64).ceil() as usize;
        count.min(self.constraint.max_concurrent_connections).max(1)
    }

    pub fn throttle_delay_ms(&self) -> u64 {
        self.constraint.throttle_delay_ms
    }
}

// ============================================================================
// Phase 4: Extended SchemaDiffPreparationOptions & SchemaDiffPreparation
// ============================================================================

impl Default for SchemaDiffPreparationOptions {
    fn default() -> Self {
        Self {
            source_tables: Vec::new(),
            target_tables: Vec::new(),
            source_details: Vec::new(),
            target_details: Vec::new(),
            source_functions: Vec::new(),
            target_functions: Vec::new(),
            source_sequences: Vec::new(),
            target_sequences: Vec::new(),
            source_rules: Vec::new(),
            target_rules: Vec::new(),
            source_owners: Vec::new(),
            target_owners: Vec::new(),
            database_type: DatabaseType::Sqlite,
            target_schema: None,
            ignore_comments: false,
            cascade_delete: false,
            compare_column_order: false,
            detect_renames: false,
            rename_threshold: 0.5,
            enable_rollback: false,
            batch_patterns: Vec::new(),
            source_dialect: None,
            target_dialect: None,
            compatibility_threshold: 0.5,
            source_permissions: Vec::new(),
            target_permissions: Vec::new(),
            shard_strategy: None,
            resource_constraint: None,
        }
    }
}

// Add new optional fields to SchemaDiffPreparationOptions
// These are added as separate impl blocks to avoid breaking existing construction sites
impl SchemaDiffPreparationOptions {
    pub fn with_rename_detection(mut self, detect: bool, threshold: f64) -> Self {
        self.detect_renames = detect;
        self.rename_threshold = threshold;
        self
    }

    pub fn with_rollback(mut self, enable: bool) -> Self {
        self.enable_rollback = enable;
        self
    }

    pub fn with_batch_patterns(mut self, patterns: Vec<BatchPattern>) -> Self {
        self.batch_patterns = patterns;
        self
    }

    pub fn with_dialects(mut self, source: Option<DialectKind>, target: Option<DialectKind>) -> Self {
        self.source_dialect = source;
        self.target_dialect = target;
        self
    }

    pub fn with_compatibility_threshold(mut self, threshold: f64) -> Self {
        self.compatibility_threshold = threshold;
        self
    }

    pub fn with_permissions(mut self, source: Vec<PermissionInfo>, target: Vec<PermissionInfo>) -> Self {
        self.source_permissions = source;
        self.target_permissions = target;
        self
    }

    pub fn with_shard_strategy(mut self, strategy: ShardStrategy) -> Self {
        self.shard_strategy = Some(strategy);
        self
    }

    pub fn with_resource_constraint(mut self, constraint: ResourceConstraint) -> Self {
        self.resource_constraint = Some(constraint);
        self
    }
}

pub fn prepare_schema_diff(options: SchemaDiffPreparationOptions) -> SchemaDiffPreparation {
    let dep_graph = DependencyGraph::build(&options.source_details, &options.source_tables);

    let mut diffs = if let Some(ref strategy) = options.shard_strategy {
        shard_diff(&options, strategy)
    } else {
        diff_schema(&options)
    };

    let rename_candidates = if options.detect_renames {
        let removed: Vec<String> = diffs.iter().filter(|d| d.diff_type == "removed").map(|d| d.name.clone()).collect();
        let added: Vec<String> = diffs.iter().filter(|d| d.diff_type == "added").map(|d| d.name.clone()).collect();
        let candidates = detect_renames(
            &removed,
            &added,
            &options.source_details,
            &options.target_details,
            options.rename_threshold,
        );

        let removed_renamed: HashSet<&str> = candidates.iter().map(|r| r.removed_name.as_str()).collect();
        let added_renamed: HashSet<&str> = candidates.iter().map(|r| r.added_name.as_str()).collect();

        diffs.retain(|d| {
            !((d.diff_type == "removed" && removed_renamed.contains(d.name.as_str()))
                || (d.diff_type == "added" && added_renamed.contains(d.name.as_str())))
        });

        for c in &candidates {
            let source_detail = options.source_details.iter().find(|d| d.name == c.removed_name);
            let target_detail = options.target_details.iter().find(|d| d.name == c.added_name);
            diffs.push(TableDiff {
                diff_type: "renamed".to_string(),
                object_type: Some("table".to_string()),
                name: c.removed_name.clone(),
                columns: None,
                indexes: None,
                foreign_keys: None,
                triggers: None,
                ddl: source_detail.and_then(|d| d.ddl.clone()),
                target_ddl: target_detail.and_then(|d| d.ddl.clone()),
                source_table_comment: None,
                target_table_comment: None,
                sync_sql: None,
            });
        }

        candidates
    } else {
        Vec::new()
    };

    let compatibility_warnings = if options.source_dialect.is_some() || options.target_dialect.is_some() {
        let src_dialect = options.source_dialect.unwrap_or(DialectKind::Mysql);
        let tgt_dialect = options.target_dialect.unwrap_or(DialectKind::Mysql);
        let mut all_warnings = Vec::new();
        for diff in &diffs {
            if diff.diff_type == "modified" {
                if let Some(source_detail) = options.source_details.iter().find(|d| d.name == diff.name) {
                    if let Some(target_detail) = options.target_details.iter().find(|d| d.name == diff.name) {
                        let (_, warnings) = diff_columns_with_compatibility(
                            &source_detail.columns,
                            &target_detail.columns,
                            options.ignore_comments,
                            options.compare_column_order,
                            src_dialect,
                            tgt_dialect,
                            options.compatibility_threshold,
                        );
                        all_warnings.extend(warnings);
                    }
                }
            }
        }
        all_warnings
    } else {
        Vec::new()
    };

    let rollback_graph = if options.enable_rollback {
        let mut graph = RollbackGraph::from_forward_diffs(&diffs, &rename_candidates, &dep_graph);
        let _ = graph.validate_consistency();
        Some(graph)
    } else {
        None
    };

    let function_diffs = diff_functions(&options.source_functions, &options.target_functions);
    let sequence_diffs = diff_sequences(&options.source_sequences, &options.target_sequences);
    let rule_diffs = diff_rules(&options.source_rules, &options.target_rules);
    let owner_diffs = diff_owners(&options.source_owners, &options.target_owners);

    for diff in &mut diffs {
        let sync_sql = generate_schema_sync_sql(
            std::slice::from_ref(diff),
            &[],
            &[],
            &[],
            &[],
            options.database_type,
            options.target_schema.as_deref(),
            options.cascade_delete,
            options.source_dialect,
        );
        if !sync_sql.is_empty() {
            diff.sync_sql = Some(sync_sql);
        }
    }

    let sync_sql = generate_schema_sync_sql(
        &diffs,
        &function_diffs,
        &sequence_diffs,
        &rule_diffs,
        &owner_diffs,
        options.database_type,
        options.target_schema.as_deref(),
        options.cascade_delete,
        options.source_dialect,
    );

    let rollback_sync_sql = rollback_graph.as_ref().map(|graph| {
        generate_rollback_sync_sql(
            graph,
            options.database_type,
            options.target_schema.as_deref(),
            options.cascade_delete,
        )
    });

    let permission_diffs = if !options.source_permissions.is_empty() || !options.target_permissions.is_empty() {
        diff_permissions(&options.source_permissions, &options.target_permissions)
    } else {
        Vec::new()
    };

    let permission_sync_sql = if !permission_diffs.is_empty() {
        Some(generate_permission_sync_sql(&permission_diffs, options.database_type, options.target_schema.as_deref()))
    } else {
        None
    };

    SchemaDiffPreparation {
        diffs,
        function_diffs,
        sequence_diffs,
        rule_diffs,
        owner_diffs,
        sync_sql,
        rollback_sync_sql,
        rename_candidates,
        rollback_graph,
        compatibility_warnings,
        permission_diffs,
        permission_sync_sql,
        dependency_graph: Some(dep_graph),
    }
}

fn diff_schema(options: &SchemaDiffPreparationOptions) -> Vec<TableDiff> {
    let source_details: HashMap<&str, &TableSchemaDetail> =
        options.source_details.iter().map(|detail| (detail.name.as_str(), detail)).collect();
    let target_details: HashMap<&str, &TableSchemaDetail> =
        options.target_details.iter().map(|detail| (detail.name.as_str(), detail)).collect();
    let source_table_comments: HashMap<&str, Option<String>> =
        options.source_tables.iter().map(|table| (table.name.as_str(), table.comment.clone())).collect();
    let target_table_comments: HashMap<&str, Option<String>> =
        options.target_tables.iter().map(|table| (table.name.as_str(), table.comment.clone())).collect();

    let source_table_names: Vec<String> = options
        .source_tables
        .iter()
        .filter(|table| !table.table_type.contains("VIEW"))
        .map(|table| table.name.clone())
        .collect();
    let target_table_names: Vec<String> = options
        .target_tables
        .iter()
        .filter(|table| !table.table_type.contains("VIEW"))
        .map(|table| table.name.clone())
        .collect();
    let source_view_names: Vec<String> = options
        .source_tables
        .iter()
        .filter(|table| table.table_type.contains("VIEW"))
        .map(|table| table.name.clone())
        .collect();
    let target_view_names: Vec<String> = options
        .target_tables
        .iter()
        .filter(|table| table.table_type.contains("VIEW"))
        .map(|table| table.name.clone())
        .collect();

    let (added, removed, common) = diff_names(&source_table_names, &target_table_names);
    let (added_views, removed_views, _) = diff_names(&source_view_names, &target_view_names);
    let mut result = Vec::new();

    for name in added {
        result.push(TableDiff {
            diff_type: "added".to_string(),
            object_type: Some("table".to_string()),
            ddl: source_details.get(name.as_str()).and_then(|detail| detail.ddl.clone()),
            target_ddl: None,
            name,
            columns: None,
            indexes: None,
            foreign_keys: None,
            triggers: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        });
    }

    for name in removed {
        let name_clone = name.clone();
        result.push(TableDiff {
            diff_type: "removed".to_string(),
            object_type: Some("table".to_string()),
            name,
            columns: None,
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: target_details.get(name_clone.as_str()).and_then(|detail| detail.ddl.clone()),
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        });
    }

    for name in added_views {
        let name_clone = name.clone();
        result.push(TableDiff {
            diff_type: "added".to_string(),
            object_type: Some("view".to_string()),
            name,
            columns: None,
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: source_details.get(name_clone.as_str()).and_then(|detail| detail.ddl.clone()),
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        });
    }

    for name in removed_views {
        let name_clone = name.clone();
        result.push(TableDiff {
            diff_type: "removed".to_string(),
            object_type: Some("view".to_string()),
            name,
            columns: None,
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: target_details.get(name_clone.as_str()).and_then(|detail| detail.ddl.clone()),
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        });
    }

    for name in common {
        let Some(source) = source_details.get(name.as_str()) else { continue };
        let Some(target) = target_details.get(name.as_str()) else { continue };
        let column_diffs = diff_columns_with_options(
            &source.columns,
            &target.columns,
            options.ignore_comments,
            options.compare_column_order,
            options.detect_renames,
            options.rename_threshold,
        );
        let index_diffs = diff_indexes(&source.indexes, &target.indexes);
        let foreign_key_diffs = diff_foreign_keys(&source.foreign_keys, &target.foreign_keys);
        let trigger_diffs = diff_triggers(&source.triggers, &target.triggers);
        let source_comment = source_table_comments.get(name.as_str()).cloned().unwrap_or(None);
        let target_comment = target_table_comments.get(name.as_str()).cloned().unwrap_or(None);
        let comment_changed = !options.ignore_comments
            && source_comment.clone().unwrap_or_default() != target_comment.clone().unwrap_or_default();

        let has_diff = !column_diffs.is_empty()
            || !index_diffs.is_empty()
            || !foreign_key_diffs.is_empty()
            || !trigger_diffs.is_empty()
            || comment_changed;

        let name_clone = name.clone();
        result.push(TableDiff {
            diff_type: if has_diff { "modified".to_string() } else { "none".to_string() },
            object_type: Some("table".to_string()),
            name,
            columns: if has_diff { (!column_diffs.is_empty()).then_some(column_diffs) } else { None },
            indexes: if has_diff { (!index_diffs.is_empty()).then_some(index_diffs) } else { None },
            foreign_keys: if has_diff { (!foreign_key_diffs.is_empty()).then_some(foreign_key_diffs) } else { None },
            triggers: if has_diff { (!trigger_diffs.is_empty()).then_some(trigger_diffs) } else { None },
            ddl: source_details.get(name_clone.as_str()).and_then(|detail| detail.ddl.clone()),
            target_ddl: target_details.get(name_clone.as_str()).and_then(|detail| detail.ddl.clone()),
            source_table_comment: if has_diff { comment_changed.then_some(source_comment) } else { None },
            target_table_comment: if has_diff { comment_changed.then_some(target_comment) } else { None },
            sync_sql: None,
        });
    }

    result.retain(|diff| diff.diff_type != "none");
    result
}

fn diff_names(source: &[String], target: &[String]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let source_set: HashSet<&str> = source.iter().map(String::as_str).collect();
    let target_set: HashSet<&str> = target.iter().map(String::as_str).collect();
    (
        source.iter().filter(|name| !target_set.contains(name.as_str())).cloned().collect(),
        target.iter().filter(|name| !source_set.contains(name.as_str())).cloned().collect(),
        source.iter().filter(|name| target_set.contains(name.as_str())).cloned().collect(),
    )
}

pub fn diff_columns(source: &[ColumnInfo], target: &[ColumnInfo]) -> Vec<ColumnDiff> {
    diff_columns_with_options(source, target, false, false, false, 0.5)
}

fn column_type_similarity_score(source_type: &str, target_type: &str) -> f64 {
    let s = ColumnType::parse(source_type).base_type.to_ascii_lowercase();
    let t = ColumnType::parse(target_type).base_type.to_ascii_lowercase();
    if s == t {
        return 1.0;
    }
    let exact_matches = [
        ("int", "integer"),
        ("integer", "int"),
        ("float", "real"),
        ("real", "float"),
        ("double", "double precision"),
        ("double precision", "double"),
        ("bool", "boolean"),
        ("boolean", "bool"),
        ("timestamp", "datetime"),
        ("datetime", "timestamp"),
    ];
    if exact_matches.contains(&(s.as_str(), t.as_str())) {
        return 1.0;
    }
    let integer_family = ["tinyint", "smallint", "mediumint", "int", "integer", "bigint", "serial", "bigserial"];
    let text_family = ["char", "varchar", "text", "tinytext", "mediumtext", "longtext", "clob", "nclob"];
    if integer_family.contains(&s.as_str()) && integer_family.contains(&t.as_str()) {
        return 0.8;
    }
    if text_family.contains(&s.as_str()) && text_family.contains(&t.as_str()) {
        return 0.8;
    }
    0.0
}

fn diff_columns_with_options(
    source: &[ColumnInfo],
    target: &[ColumnInfo],
    ignore_comments: bool,
    compare_column_order: bool,
    detect_renames: bool,
    rename_threshold: f64,
) -> Vec<ColumnDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &ColumnInfo> = target.iter().map(|column| (column.name.as_str(), column)).collect();
    let source_map: HashMap<&str, &ColumnInfo> = source.iter().map(|column| (column.name.as_str(), column)).collect();
    let target_position_map: HashMap<&str, usize> =
        target.iter().enumerate().map(|(index, column)| (column.name.as_str(), index)).collect();
    let can_compare_order = compare_column_order
        && source.len() == target.len()
        && source.iter().all(|column| target_map.contains_key(column.name.as_str()));

    for (source_index, source_column) in source.iter().enumerate() {
        if let Some(target_column) = target_map.get(source_column.name.as_str()) {
            let mut changes = Vec::new();
            if source_column.data_type.to_lowercase() != target_column.data_type.to_lowercase() {
                changes.push(format!("type: {} → {}", target_column.data_type, source_column.data_type));
            }
            if source_column.is_nullable != target_column.is_nullable {
                changes.push(format!(
                    "nullable: {} → {}",
                    if target_column.is_nullable { "YES" } else { "NO" },
                    if source_column.is_nullable { "YES" } else { "NO" }
                ));
            }
            if source_column.column_default.as_deref().unwrap_or_default()
                != target_column.column_default.as_deref().unwrap_or_default()
            {
                changes.push(format!(
                    "default: {} → {}",
                    target_column.column_default.as_deref().unwrap_or("NULL"),
                    source_column.column_default.as_deref().unwrap_or("NULL")
                ));
            }
            if !ignore_comments
                && source_column.comment.as_deref().unwrap_or_default()
                    != target_column.comment.as_deref().unwrap_or_default()
            {
                changes.push(format!(
                    "comment: {} → {}",
                    target_column.comment.as_deref().unwrap_or_default(),
                    source_column.comment.as_deref().unwrap_or_default()
                ));
            }
            if can_compare_order {
                if let Some(target_index) = target_position_map.get(source_column.name.as_str()) {
                    if source_index != *target_index {
                        changes.push(format!("order: {} → {}", *target_index + 1, source_index + 1));
                    }
                }
            }
            if !changes.is_empty() {
                diffs.push(ColumnDiff {
                    diff_type: "modified".to_string(),
                    name: source_column.name.clone(),
                    source: Some(source_column.clone()),
                    target: Some((*target_column).clone()),
                    changes,
                });
            }
        } else {
            diffs.push(ColumnDiff {
                diff_type: "added".to_string(),
                name: source_column.name.clone(),
                source: Some(source_column.clone()),
                target: None,
                changes: Vec::new(),
            });
        }
    }

    for target_column in target {
        if !source_map.contains_key(target_column.name.as_str()) {
            diffs.push(ColumnDiff {
                diff_type: "removed".to_string(),
                name: target_column.name.clone(),
                source: None,
                target: Some(target_column.clone()),
                changes: Vec::new(),
            });
        }
    }

    if detect_renames && rename_threshold > 0.0 {
        let removed_indices: Vec<usize> =
            diffs.iter().enumerate().filter(|(_, d)| d.diff_type == "removed").map(|(i, _)| i).collect();
        let added_indices: Vec<usize> =
            diffs.iter().enumerate().filter(|(_, d)| d.diff_type == "added").map(|(i, _)| i).collect();

        let mut matched_added: HashSet<usize> = HashSet::new();
        let mut matched_removed: HashSet<usize> = HashSet::new();
        let mut rename_pairs: Vec<(usize, usize, f64)> = Vec::new();

        for &ri in &removed_indices {
            if let Some(removed_col) = &diffs[ri].target {
                let mut best_score = 0.0_f64;
                let mut best_ai = None;
                for &ai in &added_indices {
                    if matched_added.contains(&ai) {
                        continue;
                    }
                    if let Some(added_col) = &diffs[ai].source {
                        let type_score = column_type_similarity_score(&removed_col.data_type, &added_col.data_type);
                        if type_score < rename_threshold {
                            continue;
                        }
                        let mut score = type_score;
                        if removed_col.is_nullable == added_col.is_nullable {
                            score *= 1.0;
                        } else {
                            score *= 0.8;
                        }
                        if score > best_score {
                            best_score = score;
                            best_ai = Some(ai);
                        }
                    }
                }
                if let Some(ai) = best_ai {
                    rename_pairs.push((ri, ai, best_score));
                    matched_removed.insert(ri);
                    matched_added.insert(ai);
                }
            }
        }

        for (ri, ai, _score) in &rename_pairs {
            let old_name = diffs[*ri].name.clone();
            let old_col = diffs[*ri].target.clone().unwrap();
            let new_col = diffs[*ai].source.clone().unwrap();
            let new_name = new_col.name.clone();

            diffs[*ri] = ColumnDiff {
                diff_type: "renamed".to_string(),
                name: new_name.clone(),
                source: Some(new_col),
                target: Some(old_col),
                changes: vec![format!("{} → {}", old_name, new_name)],
            };
            diffs[*ai] = ColumnDiff {
                diff_type: "_matched_rename".to_string(),
                name: String::new(),
                source: None,
                target: None,
                changes: Vec::new(),
            };
        }

        diffs.retain(|d| d.diff_type != "_matched_rename");
    }

    diffs
}

pub fn diff_indexes(source: &[IndexInfo], target: &[IndexInfo]) -> Vec<IndexDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &IndexInfo> = target.iter().map(|index| (index.name.as_str(), index)).collect();
    let source_map: HashMap<&str, &IndexInfo> = source.iter().map(|index| (index.name.as_str(), index)).collect();

    for source_index in source {
        if source_index.is_primary {
            continue;
        }
        let Some(target_index) = target_map.get(source_index.name.as_str()) else {
            diffs.push(IndexDiff {
                diff_type: "added".to_string(),
                name: source_index.name.clone(),
                source: Some(source_index.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_index.is_unique != target_index.is_unique {
            changes.push(format!(
                "unique: {} → {}",
                if target_index.is_unique { "YES" } else { "NO" },
                if source_index.is_unique { "YES" } else { "NO" }
            ));
        }
        if source_index.columns.join(",") != target_index.columns.join(",") {
            changes.push(format!("columns: {} → {}", target_index.columns.join(", "), source_index.columns.join(", ")));
        }
        if source_index.index_type.as_deref().unwrap_or_default()
            != target_index.index_type.as_deref().unwrap_or_default()
        {
            changes.push(format!(
                "type: {} → {}",
                target_index.index_type.as_deref().unwrap_or("default"),
                source_index.index_type.as_deref().unwrap_or("default")
            ));
        }
        if source_index.filter.as_deref().unwrap_or_default() != target_index.filter.as_deref().unwrap_or_default() {
            changes.push(format!(
                "filter: {} → {}",
                target_index.filter.as_deref().unwrap_or("none"),
                source_index.filter.as_deref().unwrap_or("none")
            ));
        }
        let source_included = source_index.included_columns.clone().unwrap_or_default();
        let target_included = target_index.included_columns.clone().unwrap_or_default();
        if source_included.join(",") != target_included.join(",") {
            changes.push(format!(
                "include: {} → {}",
                if target_included.is_empty() { "none".to_string() } else { target_included.join(", ") },
                if source_included.is_empty() { "none".to_string() } else { source_included.join(", ") }
            ));
        }
        if !changes.is_empty() {
            diffs.push(IndexDiff {
                diff_type: "modified".to_string(),
                name: source_index.name.clone(),
                source: Some(source_index.clone()),
                target: Some((*target_index).clone()),
                changes,
            });
        }
    }

    for target_index in target {
        if target_index.is_primary {
            continue;
        }
        if !source_map.contains_key(target_index.name.as_str()) {
            diffs.push(IndexDiff {
                diff_type: "removed".to_string(),
                name: target_index.name.clone(),
                source: None,
                target: Some(target_index.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

pub fn diff_foreign_keys(source: &[ForeignKeyInfo], target: &[ForeignKeyInfo]) -> Vec<ForeignKeyDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &ForeignKeyInfo> = target.iter().map(|fk| (fk.name.as_str(), fk)).collect();
    let source_map: HashMap<&str, &ForeignKeyInfo> = source.iter().map(|fk| (fk.name.as_str(), fk)).collect();

    for source_fk in source {
        let Some(target_fk) = target_map.get(source_fk.name.as_str()) else {
            diffs.push(ForeignKeyDiff {
                diff_type: "added".to_string(),
                name: source_fk.name.clone(),
                source: Some(source_fk.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_fk.column != target_fk.column {
            changes.push(format!("column: {} → {}", target_fk.column, source_fk.column));
        }
        if source_fk.ref_table != target_fk.ref_table {
            changes.push(format!("ref table: {} → {}", target_fk.ref_table, source_fk.ref_table));
        }
        if source_fk.ref_schema != target_fk.ref_schema {
            changes.push(format!(
                "ref schema: {} → {}",
                target_fk.ref_schema.as_deref().unwrap_or(""),
                source_fk.ref_schema.as_deref().unwrap_or("")
            ));
        }
        if source_fk.ref_column != target_fk.ref_column {
            changes.push(format!("ref column: {} → {}", target_fk.ref_column, source_fk.ref_column));
        }
        if !changes.is_empty() {
            diffs.push(ForeignKeyDiff {
                diff_type: "modified".to_string(),
                name: source_fk.name.clone(),
                source: Some(source_fk.clone()),
                target: Some((*target_fk).clone()),
                changes,
            });
        }
    }

    for target_fk in target {
        if !source_map.contains_key(target_fk.name.as_str()) {
            diffs.push(ForeignKeyDiff {
                diff_type: "removed".to_string(),
                name: target_fk.name.clone(),
                source: None,
                target: Some(target_fk.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

pub fn diff_triggers(source: &[TriggerInfo], target: &[TriggerInfo]) -> Vec<TriggerDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &TriggerInfo> =
        target.iter().map(|trigger| (trigger.name.as_str(), trigger)).collect();
    let source_map: HashMap<&str, &TriggerInfo> =
        source.iter().map(|trigger| (trigger.name.as_str(), trigger)).collect();

    for source_trigger in source {
        let Some(target_trigger) = target_map.get(source_trigger.name.as_str()) else {
            diffs.push(TriggerDiff {
                diff_type: "added".to_string(),
                name: source_trigger.name.clone(),
                source: Some(source_trigger.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_trigger.event != target_trigger.event {
            changes.push(format!("event: {} → {}", target_trigger.event, source_trigger.event));
        }
        if source_trigger.timing != target_trigger.timing {
            changes.push(format!("timing: {} → {}", target_trigger.timing, source_trigger.timing));
        }
        if !changes.is_empty() {
            diffs.push(TriggerDiff {
                diff_type: "modified".to_string(),
                name: source_trigger.name.clone(),
                source: Some(source_trigger.clone()),
                target: Some((*target_trigger).clone()),
                changes,
            });
        }
    }

    for target_trigger in target {
        if !source_map.contains_key(target_trigger.name.as_str()) {
            diffs.push(TriggerDiff {
                diff_type: "removed".to_string(),
                name: target_trigger.name.clone(),
                source: None,
                target: Some(target_trigger.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

fn is_mysql_like(db_type: DatabaseType) -> bool {
    matches!(
        db_type,
        DatabaseType::Mysql
            | DatabaseType::Doris
            | DatabaseType::StarRocks
            | DatabaseType::Goldendb
            | DatabaseType::Sundb
            | DatabaseType::Databend
            | DatabaseType::Gbase
    )
}

/// Normalize a function definition for comparison by:
/// - Converting CRLF to LF
/// - Collapsing all whitespace (tabs, multiple spaces) to single spaces
/// - Trimming each line and rejoining
pub(crate) fn normalize_definition(def: &str) -> String {
    def.replace("\r\n", "\n")
        .split('\n')
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn diff_functions(source: &[FunctionInfo], target: &[FunctionInfo]) -> Vec<FunctionDiff> {
    let mut diffs = Vec::new();
    // Use (name, arguments) as key to support PostgreSQL function overloading
    let target_map: HashMap<(&str, &str), &FunctionInfo> =
        target.iter().map(|f| ((f.name.as_str(), f.arguments.as_str()), f)).collect();
    let source_map: HashMap<(&str, &str), &FunctionInfo> =
        source.iter().map(|f| ((f.name.as_str(), f.arguments.as_str()), f)).collect();

    for source_fn in source {
        let key = (source_fn.name.as_str(), source_fn.arguments.as_str());
        let Some(target_fn) = target_map.get(&key) else {
            diffs.push(FunctionDiff {
                diff_type: "added".to_string(),
                name: source_fn.name.clone(),
                source: Some(source_fn.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_fn.function_type != target_fn.function_type {
            changes.push(format!("type: {} → {}", target_fn.function_type, source_fn.function_type));
        }
        if source_fn.data_type != target_fn.data_type {
            changes.push(format!("return type: {} → {}", target_fn.data_type, source_fn.data_type));
        }
        if normalize_definition(&source_fn.definition) != normalize_definition(&target_fn.definition) {
            changes.push("definition changed".to_string());
        }
        if !changes.is_empty() {
            diffs.push(FunctionDiff {
                diff_type: "modified".to_string(),
                name: source_fn.name.clone(),
                source: Some(source_fn.clone()),
                target: Some((*target_fn).clone()),
                changes,
            });
        }
    }

    for target_fn in target {
        let key = (target_fn.name.as_str(), target_fn.arguments.as_str());
        if !source_map.contains_key(&key) {
            diffs.push(FunctionDiff {
                diff_type: "removed".to_string(),
                name: target_fn.name.clone(),
                source: None,
                target: Some(target_fn.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

pub fn diff_sequences(source: &[SequenceInfo], target: &[SequenceInfo]) -> Vec<SequenceDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &SequenceInfo> = target.iter().map(|s| (s.name.as_str(), s)).collect();
    let source_map: HashMap<&str, &SequenceInfo> = source.iter().map(|s| (s.name.as_str(), s)).collect();

    for source_seq in source {
        let Some(target_seq) = target_map.get(source_seq.name.as_str()) else {
            diffs.push(SequenceDiff {
                diff_type: "added".to_string(),
                name: source_seq.name.clone(),
                source: Some(source_seq.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_seq.data_type != target_seq.data_type {
            changes.push(format!("data_type: {} → {}", target_seq.data_type, source_seq.data_type));
        }
        if source_seq.start_value != target_seq.start_value {
            changes.push(format!("start: {} → {}", target_seq.start_value, source_seq.start_value));
        }
        if source_seq.min_value != target_seq.min_value {
            changes.push(format!("min: {} → {}", target_seq.min_value, source_seq.min_value));
        }
        if source_seq.max_value != target_seq.max_value {
            changes.push(format!("max: {} → {}", target_seq.max_value, source_seq.max_value));
        }
        if source_seq.increment != target_seq.increment {
            changes.push(format!("increment: {} → {}", target_seq.increment, source_seq.increment));
        }
        if source_seq.cycle != target_seq.cycle {
            changes.push(format!("cycle: {} → {}", target_seq.cycle, source_seq.cycle));
        }
        // Only compare last_value when both sides successfully retrieved it.
        // Avoid false positives when one side lacks permission (returns None).
        if let (Some(s), Some(t)) = (&source_seq.last_value, &target_seq.last_value) {
            if s != t {
                changes.push(format!("last_value: {} → {}", t, s));
            }
        }
        if !changes.is_empty() {
            diffs.push(SequenceDiff {
                diff_type: "modified".to_string(),
                name: source_seq.name.clone(),
                source: Some(source_seq.clone()),
                target: Some((*target_seq).clone()),
                changes,
            });
        }
    }

    for target_seq in target {
        if !source_map.contains_key(target_seq.name.as_str()) {
            diffs.push(SequenceDiff {
                diff_type: "removed".to_string(),
                name: target_seq.name.clone(),
                source: None,
                target: Some(target_seq.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

pub fn diff_rules(source: &[RuleInfo], target: &[RuleInfo]) -> Vec<RuleDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &RuleInfo> = target.iter().map(|r| (r.name.as_str(), r)).collect();
    let source_map: HashMap<&str, &RuleInfo> = source.iter().map(|r| (r.name.as_str(), r)).collect();

    for source_rule in source {
        let Some(target_rule) = target_map.get(source_rule.name.as_str()) else {
            diffs.push(RuleDiff {
                diff_type: "added".to_string(),
                name: source_rule.name.clone(),
                source: Some(source_rule.clone()),
                target: None,
                changes: Vec::new(),
            });
            continue;
        };

        let mut changes = Vec::new();
        if source_rule.definition != target_rule.definition {
            changes.push("definition changed".to_string());
        }
        if !changes.is_empty() {
            diffs.push(RuleDiff {
                diff_type: "modified".to_string(),
                name: source_rule.name.clone(),
                source: Some(source_rule.clone()),
                target: Some((*target_rule).clone()),
                changes,
            });
        }
    }

    for target_rule in target {
        if !source_map.contains_key(target_rule.name.as_str()) {
            diffs.push(RuleDiff {
                diff_type: "removed".to_string(),
                name: target_rule.name.clone(),
                source: None,
                target: Some(target_rule.clone()),
                changes: Vec::new(),
            });
        }
    }

    diffs
}

pub fn diff_owners(source: &[OwnerInfo], target: &[OwnerInfo]) -> Vec<OwnerDiff> {
    let mut diffs = Vec::new();
    let target_map: HashMap<&str, &OwnerInfo> = target.iter().map(|o| (o.object_name.as_str(), o)).collect();
    let _source_map: HashMap<&str, &OwnerInfo> = source.iter().map(|o| (o.object_name.as_str(), o)).collect();

    for source_owner in source {
        let Some(target_owner) = target_map.get(source_owner.object_name.as_str()) else {
            continue; // skip added/removed objects, only compare owners for common objects
        };

        let mut changes = Vec::new();
        if source_owner.owner != target_owner.owner {
            changes.push(format!("owner: {} → {}", target_owner.owner, source_owner.owner));
        }
        if !changes.is_empty() {
            diffs.push(OwnerDiff {
                diff_type: "modified".to_string(),
                object_name: source_owner.object_name.clone(),
                source: Some(source_owner.clone()),
                target: Some((*target_owner).clone()),
                changes,
            });
        }
    }

    diffs
}

fn quote_id(name: &str, db_type: DatabaseType) -> String {
    if is_mysql_like(db_type) {
        format!("`{}`", name.replace('`', "``"))
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

fn column_def(col: &ColumnInfo, db_type: DatabaseType) -> String {
    let mut definition = format!("{} {}", quote_id(&col.name, db_type), col.data_type);
    if !col.is_nullable {
        definition.push_str(" NOT NULL");
    }
    if let Some(default) = &col.column_default {
        definition.push_str(&format!(" DEFAULT {default}"));
    }
    if is_mysql_like(db_type) {
        if let Some(comment) = &col.comment {
            definition.push_str(&format!(" COMMENT {}", comment_literal(comment)));
        }
    }
    definition
}

fn qualified_name(name: &str, db_type: DatabaseType, schema: Option<&str>) -> String {
    schema
        .map(|schema| format!("{}.{}", quote_id(schema, db_type), quote_id(name, db_type)))
        .unwrap_or_else(|| quote_id(name, db_type))
}

fn drop_index_sql(table_name: &str, index_name: &str, db_type: DatabaseType, schema: Option<&str>) -> String {
    let table = qualified_name(table_name, db_type, schema);
    let index = qualified_name(index_name, db_type, schema);
    if is_mysql_like(db_type) {
        format!("DROP INDEX {} ON {table};", quote_id(index_name, db_type))
    } else {
        format!("DROP INDEX IF EXISTS {index};")
    }
}

fn create_index_sql(table_name: &str, index: &IndexInfo, db_type: DatabaseType, schema: Option<&str>) -> String {
    let table = qualified_name(table_name, db_type, schema);
    let columns = index.columns.iter().map(|column| quote_id(column, db_type)).collect::<Vec<_>>().join(", ");
    let unique = if index.is_unique { "UNIQUE " } else { "" };
    let index_type = index.index_type.as_deref().unwrap_or_default();
    let using_clause = if !index_type.is_empty() && db_type == DatabaseType::Postgres {
        format!(" USING {index_type}")
    } else {
        String::new()
    };
    let type_prefix = if !index_type.is_empty() && db_type == DatabaseType::SqlServer {
        format!("{index_type} ")
    } else {
        String::new()
    };
    let mysql_using =
        if !index_type.is_empty() && is_mysql_like(db_type) { format!(" USING {index_type}") } else { String::new() };
    let included_columns = index.included_columns.clone().unwrap_or_default();
    let include_clause =
        if !included_columns.is_empty() && matches!(db_type, DatabaseType::Postgres | DatabaseType::SqlServer) {
            format!(
                " INCLUDE ({})",
                included_columns.iter().map(|column| quote_id(column, db_type)).collect::<Vec<_>>().join(", ")
            )
        } else {
            String::new()
        };
    let supports_where = matches!(db_type, DatabaseType::Postgres | DatabaseType::SqlServer | DatabaseType::Sqlite);
    let filter = if supports_where { index.filter.as_deref().unwrap_or_default() } else { "" };
    let filter_clause = if filter.is_empty() { String::new() } else { format!(" WHERE {filter}") };
    let comment = index.comment.as_deref().unwrap_or("");
    let comment_clause = if !comment.trim().is_empty() && is_mysql_like(db_type) {
        format!(" COMMENT {}", comment_literal(comment))
    } else {
        String::new()
    };
    if is_mysql_like(db_type) {
        format!(
            "CREATE {unique}{type_prefix}INDEX {}{mysql_using} ON {table} ({columns}){comment_clause};",
            quote_id(&index.name, db_type)
        )
    } else {
        format!(
            "CREATE {unique}{type_prefix}INDEX {} ON {table}{using_clause} ({columns}){include_clause}{filter_clause};",
            quote_id(&index.name, db_type)
        )
    }
}

fn drop_foreign_key_sql(table_name: &str, fk_name: &str, db_type: DatabaseType, schema: Option<&str>) -> String {
    let table = qualified_name(table_name, db_type, schema);
    let fk = quote_id(fk_name, db_type);
    if is_mysql_like(db_type) {
        format!("ALTER TABLE {table} DROP FOREIGN KEY {fk};")
    } else {
        format!("ALTER TABLE {table} DROP CONSTRAINT {fk};")
    }
}

fn add_foreign_key_sql(table_name: &str, fk: &ForeignKeyInfo, db_type: DatabaseType, schema: Option<&str>) -> String {
    let table = qualified_name(table_name, db_type, schema);
    format!(
        "ALTER TABLE {table} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({});",
        quote_id(&fk.name, db_type),
        quote_id(&fk.column, db_type),
        quote_id(&fk.ref_table, db_type),
        quote_id(&fk.ref_column, db_type)
    )
}

fn drop_object_sql(diff: &TableDiff, db_type: DatabaseType, schema: Option<&str>, cascade: &str) -> String {
    let object_type = if diff.object_type.as_deref() == Some("view") { "VIEW" } else { "TABLE" };
    format!("DROP {object_type} IF EXISTS {}{cascade};", qualified_name(&diff.name, db_type, schema))
}

fn comment_literal(comment: &str) -> String {
    format!("'{}'", comment.replace('\'', "''"))
}

fn column_comment_sql(
    table_name: &str,
    column_name: &str,
    comment: &str,
    db_type: DatabaseType,
    schema: Option<&str>,
) -> String {
    if is_mysql_like(db_type) {
        return format!(
            "-- Column comment for {column_name}: use ALTER TABLE ... MODIFY COLUMN to set comment in MySQL"
        );
    }
    let table = qualified_name(table_name, db_type, schema);
    format!("COMMENT ON COLUMN {table}.{} IS {};", quote_id(column_name, db_type), comment_literal(comment))
}

fn table_comment_sql(table_name: &str, comment: &str, db_type: DatabaseType, schema: Option<&str>) -> String {
    let table = qualified_name(table_name, db_type, schema);
    if is_mysql_like(db_type) {
        format!("ALTER TABLE {table} COMMENT = {};", comment_literal(comment))
    } else {
        format!("COMMENT ON TABLE {table} IS {};", comment_literal(comment))
    }
}

#[allow(clippy::too_many_arguments)]
pub fn generate_schema_sync_sql(
    diffs: &[TableDiff],
    function_diffs: &[FunctionDiff],
    sequence_diffs: &[SequenceDiff],
    rule_diffs: &[RuleDiff],
    owner_diffs: &[OwnerDiff],
    db_type: DatabaseType,
    schema: Option<&str>,
    cascade_delete: bool,
    source_dialect: Option<DialectKind>,
) -> String {
    let mut lines = Vec::new();
    let is_mysql = is_mysql_like(db_type);
    let cascade = if cascade_delete { " CASCADE" } else { "" };

    let type_matrix = source_dialect.map(|src| {
        let target_dialect = DialectKind::from_database_type(db_type);
        crate::sql_dialect::descriptor::TypeMappingMatrix::for_dialects(src, target_dialect)
    });
    let map_type = |source_type: &str| -> String {
        type_matrix.as_ref().map_or_else(|| source_type.to_string(), |m| m.convert_type(source_type).0)
    };

    for diff in diffs {
        let table = qualified_name(&diff.name, db_type, schema);

        if diff.diff_type == "added" && diff.ddl.is_some() {
            lines.push(format!("-- Create {}: {}", diff.object_type.as_deref().unwrap_or("table"), diff.name));
            lines.push(format!("{};", diff.ddl.as_deref().unwrap_or_default()));
            lines.push(String::new());
            continue;
        }

        if diff.diff_type == "added" && diff.object_type.as_deref() == Some("view") {
            lines.push(format!("-- View exists only in source: {}", diff.name));
            lines.push("-- Source view definition is not available from this driver yet.".to_string());
            lines.push(String::new());
            continue;
        }

        if diff.diff_type == "removed" {
            lines.push(format!("-- Drop {}: {}", diff.object_type.as_deref().unwrap_or("table"), diff.name));
            lines.push(drop_object_sql(diff, db_type, schema, cascade));
            lines.push(String::new());
            continue;
        }

        if diff.diff_type != "modified" {
            continue;
        }

        let mut parts = Vec::new();
        if let Some(foreign_keys) = &diff.foreign_keys {
            for fk in foreign_keys {
                if fk.diff_type == "removed" || fk.diff_type == "modified" {
                    lines.push(drop_foreign_key_sql(&diff.name, &fk.name, db_type, schema));
                }
            }
        }

        if let Some(columns) = &diff.columns {
            let convert_col =
                |col: &ColumnInfo| -> ColumnInfo { ColumnInfo { data_type: map_type(&col.data_type), ..col.clone() } };
            for column in columns {
                match column.diff_type.as_str() {
                    "added" => {
                        if let Some(source) = &column.source {
                            parts.push(format!("  ADD COLUMN {}", column_def(&convert_col(source), db_type)));
                        }
                    }
                    "removed" => {
                        parts.push(format!("  DROP COLUMN {}", quote_id(&column.name, db_type)));
                    }
                    "modified" => {
                        if let Some(source) = &column.source {
                            let mapped = convert_col(source);
                            if is_mysql {
                                if column.changes.iter().any(|change| !change.starts_with("order:")) {
                                    parts.push(format!("  MODIFY COLUMN {}", column_def(&mapped, db_type)));
                                }
                            } else {
                                let name = quote_id(&column.name, db_type);
                                if column.changes.iter().any(|change| change.starts_with("type:")) {
                                    parts.push(format!("  ALTER COLUMN {name} TYPE {}", mapped.data_type));
                                }
                                if column.changes.iter().any(|change| change.starts_with("nullable:")) {
                                    parts.push(if source.is_nullable {
                                        format!("  ALTER COLUMN {name} DROP NOT NULL")
                                    } else {
                                        format!("  ALTER COLUMN {name} SET NOT NULL")
                                    });
                                }
                                if column.changes.iter().any(|change| change.starts_with("default:")) {
                                    parts.push(if let Some(default) = &source.column_default {
                                        format!("  ALTER COLUMN {name} SET DEFAULT {default}")
                                    } else {
                                        format!("  ALTER COLUMN {name} DROP DEFAULT")
                                    });
                                }
                            }
                        }
                    }
                    "renamed" => {
                        if let (Some(source), Some(target_col)) = (&column.source, &column.target) {
                            let mapped = convert_col(source);
                            if is_mysql {
                                let old_name = quote_id(&target_col.name, db_type);
                                parts.push(format!("  CHANGE COLUMN {} {}", old_name, column_def(&mapped, db_type)));
                            } else if db_type == DatabaseType::Postgres {
                                let old_name = quote_id(&target_col.name, db_type);
                                let new_name = quote_id(&column.name, db_type);
                                parts.push(format!("  RENAME COLUMN {old_name} TO {new_name}"));
                                if source.data_type.to_lowercase() != target_col.data_type.to_lowercase() {
                                    parts.push(format!("  ALTER COLUMN {new_name} TYPE {}", mapped.data_type));
                                }
                                if source.is_nullable != target_col.is_nullable {
                                    let action = if source.is_nullable { "DROP NOT NULL" } else { "SET NOT NULL" };
                                    parts.push(format!("  ALTER COLUMN {new_name} {action}"));
                                }
                            } else if db_type == DatabaseType::H2 {
                                let old_name = quote_id(&target_col.name, db_type);
                                let new_name = quote_id(&column.name, db_type);
                                parts.push(format!("  ALTER COLUMN {old_name} RENAME TO {new_name}"));
                                if source.data_type.to_lowercase() != target_col.data_type.to_lowercase() {
                                    parts.push(format!("  ALTER COLUMN {new_name} SET DATA TYPE {}", mapped.data_type));
                                }
                            } else if db_type == DatabaseType::SqlServer {
                                let target_table = qualified_name(&diff.name, db_type, schema);
                                let full_obj_path = format!("{target_table}.{}", quote_id(&target_col.name, db_type));
                                parts.push(format!(
                                    "  EXEC sp_rename '{}', '{}', 'COLUMN';",
                                    full_obj_path.replace('\'', "''"),
                                    column.name.replace('\'', "''")
                                ));
                            } else {
                                let old_name = quote_id(&target_col.name, db_type);
                                let new_name = quote_id(&column.name, db_type);
                                parts.push(format!("  RENAME COLUMN {old_name} TO {new_name}"));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if !parts.is_empty() {
            lines.push(format!("-- Alter table: {}", diff.name));
            if is_mysql {
                lines.push(format!("ALTER TABLE {table}"));
                lines.push(format!("{};", parts.join(",\n")));
            } else {
                for part in parts {
                    lines.push(format!("ALTER TABLE {table}{part};"));
                }
            }
            lines.push(String::new());
        }

        if !is_mysql {
            if let Some(columns) = &diff.columns {
                for column in columns {
                    if let Some(source) = &column.source {
                        if column.changes.iter().any(|change| change.starts_with("comment:")) {
                            lines.push(column_comment_sql(
                                &diff.name,
                                &column.name,
                                source.comment.as_deref().unwrap_or_default(),
                                db_type,
                                schema,
                            ));
                        }
                        if column.diff_type == "added" {
                            if let Some(comment) = &source.comment {
                                lines.push(column_comment_sql(&diff.name, &column.name, comment, db_type, schema));
                            }
                        }
                        if column.diff_type == "renamed" {
                            if let Some(comment) = &source.comment {
                                lines.push(column_comment_sql(&diff.name, &column.name, comment, db_type, schema));
                            }
                        }
                    }
                }
            }
        }

        if diff.source_table_comment.is_some() && diff.source_table_comment != diff.target_table_comment {
            let comment = diff.source_table_comment.as_ref().and_then(|comment| comment.as_deref()).unwrap_or_default();
            lines.push(table_comment_sql(&diff.name, comment, db_type, schema));
        }

        if let Some(indexes) = &diff.indexes {
            for index in indexes {
                match index.diff_type.as_str() {
                    "added" => {
                        if let Some(source) = &index.source {
                            lines.push(create_index_sql(&diff.name, source, db_type, schema));
                        }
                    }
                    "removed" => lines.push(drop_index_sql(&diff.name, &index.name, db_type, schema)),
                    "modified" => {
                        if let Some(source) = &index.source {
                            lines.push(drop_index_sql(&diff.name, &index.name, db_type, schema));
                            lines.push(create_index_sql(&diff.name, source, db_type, schema));
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(foreign_keys) = &diff.foreign_keys {
            for fk in foreign_keys {
                if fk.diff_type == "added" || fk.diff_type == "modified" {
                    if let Some(source) = &fk.source {
                        lines.push(add_foreign_key_sql(&diff.name, source, db_type, schema));
                    }
                }
            }
        }

        if let Some(triggers) = &diff.triggers {
            for trigger in triggers {
                lines.push(format!(
                    "-- Trigger {}: {} on {}; review trigger definition manually.",
                    trigger.diff_type, trigger.name, diff.name
                ));
            }
        }

        if diff.indexes.as_ref().is_some_and(|indexes| !indexes.is_empty())
            || diff.foreign_keys.as_ref().is_some_and(|foreign_keys| !foreign_keys.is_empty())
            || diff.triggers.as_ref().is_some_and(|triggers| !triggers.is_empty())
        {
            lines.push(String::new());
        }

        if db_type == DatabaseType::Sqlite
            && diff.foreign_keys.as_ref().is_some_and(|foreign_keys| !foreign_keys.is_empty())
        {
            lines.push(format!("-- SQLite foreign key synchronization may require table rebuild for: {}", diff.name));
            lines.push(String::new());
        }
    }

    // Function diffs
    if !function_diffs.is_empty() {
        lines.push(String::new());
        lines.push("-- Functions".to_string());
        for diff in function_diffs {
            match diff.diff_type.as_str() {
                "added" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Create function: {}", diff.name));
                        lines.push(format!(
                            "CREATE OR REPLACE FUNCTION {} {};",
                            qualified_name(&diff.name, db_type, schema),
                            source.definition
                        ));
                    }
                }
                "removed" => {
                    lines.push(format!("-- Drop function: {}", diff.name));
                    lines.push(format!(
                        "DROP FUNCTION IF EXISTS {}{cascade};",
                        qualified_name(&diff.name, db_type, schema)
                    ));
                }
                "modified" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Alter function: {}", diff.name));
                        lines.push(format!(
                            "CREATE OR REPLACE FUNCTION {} {};",
                            qualified_name(&diff.name, db_type, schema),
                            source.definition
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Sequence diffs
    if !sequence_diffs.is_empty() {
        lines.push(String::new());
        lines.push("-- Sequences".to_string());
        for diff in sequence_diffs {
            match diff.diff_type.as_str() {
                "added" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Create sequence: {}", diff.name));
                        lines.push(format!(
                            "CREATE SEQUENCE {} AS {} START WITH {} INCREMENT BY {} MINVALUE {} MAXVALUE {} {};",
                            qualified_name(&diff.name, db_type, schema),
                            source.data_type,
                            source.start_value,
                            source.increment,
                            source.min_value,
                            source.max_value,
                            if source.cycle { "CYCLE" } else { "NO CYCLE" }
                        ));
                    }
                }
                "removed" => {
                    lines.push(format!("-- Drop sequence: {}", diff.name));
                    lines.push(format!("DROP SEQUENCE {}{cascade};", qualified_name(&diff.name, db_type, schema)));
                }
                "modified" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Alter sequence: {}", diff.name));
                        lines.push(format!(
                            "ALTER SEQUENCE {} AS {} START WITH {} INCREMENT BY {} MINVALUE {} MAXVALUE {} {};",
                            qualified_name(&diff.name, db_type, schema),
                            source.data_type,
                            source.start_value,
                            source.increment,
                            source.min_value,
                            source.max_value,
                            if source.cycle { "CYCLE" } else { "NO CYCLE" }
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Rule diffs
    if !rule_diffs.is_empty() {
        lines.push(String::new());
        lines.push("-- Rules".to_string());
        for diff in rule_diffs {
            match diff.diff_type.as_str() {
                "added" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Create rule: {}", diff.name));
                        lines.push(source.definition.clone());
                    }
                }
                "removed" => {
                    lines.push(format!("-- Drop rule: {}", diff.name));
                    if let Some(source) = &diff.source {
                        lines.push(format!(
                            "DROP RULE IF EXISTS {} ON {};",
                            diff.name,
                            qualified_name(&source.table_name, db_type, schema)
                        ));
                    }
                }
                "modified" => {
                    if let Some(source) = &diff.source {
                        lines.push(format!("-- Alter rule: {}", diff.name));
                        lines.push(format!(
                            "DROP RULE IF EXISTS {} ON {};",
                            diff.name,
                            qualified_name(&source.table_name, db_type, schema)
                        ));
                        lines.push(source.definition.clone());
                    }
                }
                _ => {}
            }
        }
    }

    // Owner diffs
    if !owner_diffs.is_empty() {
        lines.push(String::new());
        lines.push("-- Owners".to_string());
        for diff in owner_diffs {
            if let (Some(source), Some(_target)) = (&diff.source, &diff.target) {
                let object_type = match source.object_type.as_str() {
                    "TABLE" => "TABLE",
                    "VIEW" => "VIEW",
                    "SEQUENCE" => "SEQUENCE",
                    _ => "TABLE",
                };
                lines.push(format!(
                    "ALTER {object_type} {} OWNER TO {};",
                    qualified_name(&diff.object_name, db_type, schema),
                    source.owner
                ));
            }
        }
    }

    lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(overrides: IndexInfo) -> IndexInfo {
        IndexInfo {
            name: if overrides.name.is_empty() { "idx_users_email".to_string() } else { overrides.name },
            columns: if overrides.columns.is_empty() { vec!["email".to_string()] } else { overrides.columns },
            is_unique: overrides.is_unique,
            is_primary: overrides.is_primary,
            filter: overrides.filter,
            index_type: overrides.index_type,
            included_columns: overrides.included_columns,
            comment: overrides.comment,
        }
    }

    fn foreign_key(overrides: ForeignKeyInfo) -> ForeignKeyInfo {
        ForeignKeyInfo {
            name: if overrides.name.is_empty() { "orders_user_id_fk".to_string() } else { overrides.name },
            column: if overrides.column.is_empty() { "user_id".to_string() } else { overrides.column },
            ref_schema: overrides.ref_schema,
            ref_table: if overrides.ref_table.is_empty() { "users".to_string() } else { overrides.ref_table },
            ref_column: if overrides.ref_column.is_empty() { "id".to_string() } else { overrides.ref_column },
            on_update: overrides.on_update,
            on_delete: overrides.on_delete,
        }
    }

    fn column(name: &str, data_type: &str, comment: Option<&str>) -> ColumnInfo {
        ColumnInfo {
            name: name.to_string(),
            data_type: data_type.to_string(),
            is_nullable: false,
            column_default: None,
            is_primary_key: false,
            extra: None,
            comment: comment.map(str::to_string),
            numeric_precision: None,
            numeric_scale: None,
            character_maximum_length: None,
        }
    }

    #[test]
    fn ignores_column_order_when_option_is_disabled() {
        let diffs = diff_columns_with_options(
            &[column("id", "int", None), column("name", "varchar(64)", None), column("status", "varchar(16)", None)],
            &[column("status", "varchar(16)", None), column("id", "int", None), column("name", "varchar(64)", None)],
            false,
            false,
            false,
            0.5,
        );

        assert!(diffs.is_empty());
    }

    #[test]
    fn detects_column_order_when_option_is_enabled() {
        let diffs = diff_columns_with_options(
            &[column("id", "int", None), column("name", "varchar(64)", None), column("status", "varchar(16)", None)],
            &[column("status", "varchar(16)", None), column("id", "int", None), column("name", "varchar(64)", None)],
            false,
            true,
            false,
            0.5,
        );

        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs[0].changes, vec!["order: 2 → 1"]);
    }

    #[test]
    fn detects_column_rename_with_same_type() {
        let source = vec![
            column("id", "int", None),
            column("name2", "varchar(120)", None),
            column("del_flag", "tinyint", None),
            column("create_at", "datetime", None),
        ];
        let target =
            vec![column("id", "int", None), column("name", "varchar(120)", None), column("del_flag", "tinyint", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, true, 0.5);
        let renamed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "renamed").collect();
        let added: Vec<_> = diffs.iter().filter(|d| d.diff_type == "added").collect();
        let removed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "removed").collect();
        assert_eq!(renamed.len(), 1, "should detect one renamed column");
        assert_eq!(renamed[0].name, "name2");
        assert_eq!(renamed[0].changes, vec!["name → name2"]);
        assert_eq!(added.len(), 1, "should have one truly added column (create_at)");
        assert_eq!(added[0].name, "create_at");
        assert!(removed.is_empty(), "should have no removed columns");
    }

    #[test]
    fn detects_column_rename_with_compatible_type() {
        let source = vec![column("col_a", "varchar(64)", None), column("col_b", "int", None)];
        let target = vec![column("col_a_old", "varchar(100)", None), column("col_b", "int", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, true, 0.5);
        let renamed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "renamed").collect();
        assert_eq!(renamed.len(), 1, "should detect rename across varchar family");
        assert_eq!(renamed[0].changes, vec!["col_a_old → col_a"]);
    }

    #[test]
    fn no_rename_detection_when_disabled() {
        let source = vec![
            column("id", "int", None),
            column("name2", "varchar(120)", None),
            column("create_at", "datetime", None),
        ];
        let target = vec![column("id", "int", None), column("name", "varchar(120)", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, false, 0.5);
        let renamed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "renamed").collect();
        let added: Vec<_> = diffs.iter().filter(|d| d.diff_type == "added").collect();
        let removed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "removed").collect();
        assert!(renamed.is_empty(), "should not detect renames when disabled");
        assert_eq!(added.len(), 2);
        assert_eq!(removed.len(), 1);
    }

    #[test]
    fn rename_not_detected_with_unrelated_types() {
        let source = vec![column("col_a", "varchar(120)", None), column("col_b", "int", None)];
        let target = vec![column("col_old", "int", None), column("col_b", "int", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, true, 0.5);
        let renamed: Vec<_> = diffs.iter().filter(|d| d.diff_type == "renamed").collect();
        assert!(renamed.is_empty(), "should not rename across unrelated types");
    }

    #[test]
    fn rename_with_rollback_graph_inversion() {
        let source = vec![column("id", "int", None), column("new_name", "varchar(120)", None)];
        let target = vec![column("id", "int", None), column("old_name", "varchar(120)", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, true, 0.5);
        let inverted = RollbackGraph::invert_columns(&diffs);
        let renamed_inv: Vec<_> = inverted.iter().filter(|d| d.diff_type == "renamed").collect();
        assert_eq!(renamed_inv.len(), 1, "inverted rename should exist");
        assert_eq!(renamed_inv[0].name, "old_name");
        assert_eq!(renamed_inv[0].changes, vec!["new_name → old_name"]);
    }

    #[test]
    fn column_rename_generates_change_column_sql_for_mysql() {
        let source = vec![
            column("id", "int", None),
            column("name2", "varchar(120)", None),
            column("del_flag", "tinyint", None),
            column("create_at", "datetime", None),
        ];
        let target =
            vec![column("id", "int", None), column("name", "varchar(120)", None), column("del_flag", "tinyint", None)];
        let diffs = diff_columns_with_options(&source, &target, false, false, true, 0.5);
        let table_diff = TableDiff {
            diff_type: "modified".to_string(),
            object_type: Some("table".to_string()),
            name: "tb_user".to_string(),
            columns: Some(diffs),
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        };
        let sql = generate_schema_sync_sql(&[table_diff], &[], &[], &[], &[], DatabaseType::Mysql, None, false, None);
        assert!(sql.contains("CHANGE COLUMN `name`"), "should generate CHANGE COLUMN for rename: {sql}");
        assert!(sql.contains("ADD COLUMN `create_at`"), "should generate ADD COLUMN for new column: {sql}");
        assert!(!sql.contains("DROP COLUMN"), "should not DROP COLUMN: {sql}");
    }

    #[test]
    fn detects_modified_indexes_not_only_added_or_removed_indexes() {
        let diffs = diff_indexes(
            &[index(IndexInfo {
                name: "idx_orders_status".to_string(),
                columns: vec!["status".to_string(), "created_at".to_string()],
                is_unique: false,
                is_primary: false,
                filter: None,
                index_type: None,
                included_columns: None,
                comment: None,
            })],
            &[index(IndexInfo {
                name: "idx_orders_status".to_string(),
                columns: vec!["status".to_string()],
                is_unique: true,
                is_primary: false,
                filter: None,
                index_type: None,
                included_columns: None,
                comment: None,
            })],
        );

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, "modified");
        assert_eq!(diffs[0].changes, vec!["unique: YES → NO", "columns: status → status, created_at"]);
    }

    #[test]
    fn detects_foreign_key_additions_removals_and_target_changes() {
        let diffs = diff_foreign_keys(
            &[
                foreign_key(ForeignKeyInfo {
                    name: "orders_user_id_fk".to_string(),
                    column: String::new(),
                    ref_schema: None,
                    ref_table: String::new(),
                    ref_column: String::new(),
                    on_update: None,
                    on_delete: None,
                }),
                foreign_key(ForeignKeyInfo {
                    name: "orders_account_id_fk".to_string(),
                    column: "account_id".to_string(),
                    ref_schema: None,
                    ref_table: "accounts".to_string(),
                    ref_column: String::new(),
                    on_update: None,
                    on_delete: None,
                }),
            ],
            &[
                foreign_key(ForeignKeyInfo {
                    name: "orders_user_id_fk".to_string(),
                    column: String::new(),
                    ref_schema: None,
                    ref_table: "members".to_string(),
                    ref_column: String::new(),
                    on_update: None,
                    on_delete: None,
                }),
                foreign_key(ForeignKeyInfo {
                    name: "orders_region_id_fk".to_string(),
                    column: "region_id".to_string(),
                    ref_schema: None,
                    ref_table: "regions".to_string(),
                    ref_column: String::new(),
                    on_update: None,
                    on_delete: None,
                }),
            ],
        );

        let summary: Vec<_> = diffs.iter().map(|diff| (diff.diff_type.as_str(), diff.name.as_str())).collect();
        assert_eq!(
            summary,
            vec![
                ("modified", "orders_user_id_fk"),
                ("added", "orders_account_id_fk"),
                ("removed", "orders_region_id_fk"),
            ]
        );
    }

    #[test]
    fn generates_sync_sql_for_index_and_foreign_key_changes() {
        let diffs = vec![TableDiff {
            diff_type: "modified".to_string(),
            object_type: None,
            name: "orders".to_string(),
            columns: None,
            indexes: Some(vec![IndexDiff {
                diff_type: "modified".to_string(),
                name: "idx_orders_status".to_string(),
                source: Some(index(IndexInfo {
                    name: "idx_orders_status".to_string(),
                    columns: vec!["status".to_string(), "created_at".to_string()],
                    is_unique: true,
                    is_primary: false,
                    filter: None,
                    index_type: None,
                    included_columns: None,
                    comment: None,
                })),
                target: None,
                changes: Vec::new(),
            }]),
            foreign_keys: Some(vec![ForeignKeyDiff {
                diff_type: "modified".to_string(),
                name: "orders_user_id_fk".to_string(),
                source: Some(foreign_key(ForeignKeyInfo {
                    name: "orders_user_id_fk".to_string(),
                    column: String::new(),
                    ref_schema: None,
                    ref_table: "users".to_string(),
                    ref_column: String::new(),
                    on_update: None,
                    on_delete: None,
                })),
                target: None,
                changes: Vec::new(),
            }]),
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        }];

        assert_eq!(
            generate_schema_sync_sql(&diffs, &[], &[], &[], &[], DatabaseType::Postgres, None, false, None),
            [
                "ALTER TABLE \"orders\" DROP CONSTRAINT \"orders_user_id_fk\";",
                "DROP INDEX IF EXISTS \"idx_orders_status\";",
                "CREATE UNIQUE INDEX \"idx_orders_status\" ON \"orders\" (\"status\", \"created_at\");",
                "ALTER TABLE \"orders\" ADD CONSTRAINT \"orders_user_id_fk\" FOREIGN KEY (\"user_id\") REFERENCES \"users\" (\"id\");",
            ]
            .join("\n")
        );
    }

    #[test]
    fn mysql_column_comment_changes_generate_modify_column_sql() {
        let diffs = vec![TableDiff {
            diff_type: "modified".to_string(),
            object_type: None,
            name: "users".to_string(),
            columns: Some(vec![ColumnDiff {
                diff_type: "modified".to_string(),
                name: "name".to_string(),
                source: Some(column("name", "varchar(64)", Some("用户姓名"))),
                target: Some(column("name", "varchar(64)", Some("Name"))),
                changes: vec!["comment: Name → 用户姓名".to_string()],
            }]),
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: Some(Some("用户表".to_string())),
            target_table_comment: Some(Some("Users".to_string())),
            sync_sql: None,
        }];

        assert_eq!(
            generate_schema_sync_sql(&diffs, &[], &[], &[], &[], DatabaseType::Mysql, None, false, None),
            [
                "-- Alter table: users",
                "ALTER TABLE `users`",
                "  MODIFY COLUMN `name` varchar(64) NOT NULL COMMENT '用户姓名';",
                "",
                "ALTER TABLE `users` COMMENT = '用户表';",
            ]
            .join("\n")
        );
    }

    #[test]
    fn ignore_comments_skips_column_and_table_comment_diffs() {
        let options = SchemaDiffPreparationOptions {
            source_tables: vec![TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: Some("用户表".to_string()),
                parent_schema: None,
                parent_name: None,
            }],
            target_tables: vec![TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: Some("Users".to_string()),
                parent_schema: None,
                parent_name: None,
            }],
            source_details: vec![TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![column("name", "varchar(64)", Some("用户姓名"))],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                triggers: Vec::new(),
                ddl: None,
            }],
            target_details: vec![TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![column("name", "varchar(64)", Some("Name"))],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                triggers: Vec::new(),
                ddl: None,
            }],
            source_functions: Vec::new(),
            target_functions: Vec::new(),
            source_sequences: Vec::new(),
            target_sequences: Vec::new(),
            source_rules: Vec::new(),
            target_rules: Vec::new(),
            source_owners: Vec::new(),
            target_owners: Vec::new(),
            database_type: DatabaseType::Mysql,
            target_schema: None,
            ignore_comments: true,
            cascade_delete: false,
            compare_column_order: false,
            ..Default::default()
        };

        let result = prepare_schema_diff(options);
        assert!(result.diffs.is_empty());
        assert!(result.sync_sql.is_empty());
    }

    #[test]
    fn prepare_schema_diff_attaches_per_table_sync_sql() {
        let options = SchemaDiffPreparationOptions {
            source_tables: vec![TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            }],
            target_tables: vec![TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            }],
            source_details: vec![TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![column("name", "varchar(128)", None)],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                triggers: Vec::new(),
                ddl: Some("CREATE TABLE `users` (`name` varchar(128));".to_string()),
            }],
            target_details: vec![TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![column("name", "varchar(64)", None)],
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                triggers: Vec::new(),
                ddl: Some("CREATE TABLE `users` (`name` varchar(64));".to_string()),
            }],
            source_functions: Vec::new(),
            target_functions: Vec::new(),
            source_sequences: Vec::new(),
            target_sequences: Vec::new(),
            source_rules: Vec::new(),
            target_rules: Vec::new(),
            source_owners: Vec::new(),
            target_owners: Vec::new(),
            database_type: DatabaseType::Mysql,
            target_schema: None,
            ignore_comments: false,
            cascade_delete: false,
            compare_column_order: false,
            ..Default::default()
        };

        let result = prepare_schema_diff(options);
        let table_sync_sql = result.diffs[0].sync_sql.as_deref().unwrap_or_default();

        assert!(table_sync_sql.contains("ALTER TABLE `users`"));
        assert!(!table_sync_sql.contains("CREATE TABLE"));
    }

    #[test]
    fn qualifies_generated_schema_sync_sql_with_target_schema() {
        let diffs = vec![TableDiff {
            diff_type: "modified".to_string(),
            object_type: None,
            name: "orders".to_string(),
            columns: Some(vec![ColumnDiff {
                diff_type: "added".to_string(),
                name: "status".to_string(),
                source: Some(ColumnInfo {
                    name: "status".to_string(),
                    data_type: "text".to_string(),
                    is_nullable: true,
                    column_default: None,
                    is_primary_key: false,
                    extra: None,
                    comment: None,
                    numeric_precision: None,
                    numeric_scale: None,
                    character_maximum_length: None,
                }),
                target: None,
                changes: Vec::new(),
            }]),
            indexes: Some(vec![IndexDiff {
                diff_type: "added".to_string(),
                name: "idx_orders_status".to_string(),
                source: Some(index(IndexInfo {
                    name: "idx_orders_status".to_string(),
                    columns: vec!["status".to_string()],
                    is_unique: false,
                    is_primary: false,
                    filter: None,
                    index_type: None,
                    included_columns: None,
                    comment: None,
                })),
                target: None,
                changes: Vec::new(),
            }]),
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        }];

        assert_eq!(
            generate_schema_sync_sql(&diffs, &[], &[], &[], &[], DatabaseType::Postgres, Some("sales"), false, None),
            [
                "-- Alter table: orders",
                "ALTER TABLE \"sales\".\"orders\"  ADD COLUMN \"status\" text;",
                "",
                "CREATE INDEX \"idx_orders_status\" ON \"sales\".\"orders\" (\"status\");",
            ]
            .join("\n")
        );
    }

    // ========================================================================
    // Phase 4.1: Dependency Graph Tests
    // ========================================================================

    #[test]
    fn dependency_graph_builds_dag_from_foreign_keys() {
        let details = vec![
            TableSchemaDetail {
                name: "orders".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![ForeignKeyInfo {
                    name: "fk_orders_users".to_string(),
                    column: "user_id".to_string(),
                    ref_schema: None,
                    ref_table: "users".to_string(),
                    ref_column: "id".to_string(),
                    on_update: None,
                    on_delete: None,
                }],
                triggers: vec![],
                ddl: None,
            },
            TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                triggers: vec![],
                ddl: None,
            },
        ];
        let tables = vec![
            TableInfo {
                name: "orders".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            },
            TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            },
        ];

        let graph = DependencyGraph::build(&details, &tables);
        assert_eq!(graph.nodes.len(), 2);
        assert!(graph.nodes["orders"].depends_on.contains(&"users".to_string()));
        assert_eq!(graph.nodes["orders"].depends_on.len(), 1);
        assert_eq!(graph.nodes["users"].depends_on.len(), 0);
    }

    #[test]
    fn dependency_graph_topological_sort_drop_order() {
        let details = vec![
            TableSchemaDetail {
                name: "order_items".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![ForeignKeyInfo {
                    name: "fk_items_orders".to_string(),
                    column: "order_id".to_string(),
                    ref_schema: None,
                    ref_table: "orders".to_string(),
                    ref_column: "id".to_string(),
                    on_update: None,
                    on_delete: None,
                }],
                triggers: vec![],
                ddl: None,
            },
            TableSchemaDetail {
                name: "orders".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![ForeignKeyInfo {
                    name: "fk_orders_users".to_string(),
                    column: "user_id".to_string(),
                    ref_schema: None,
                    ref_table: "users".to_string(),
                    ref_column: "id".to_string(),
                    on_update: None,
                    on_delete: None,
                }],
                triggers: vec![],
                ddl: None,
            },
            TableSchemaDetail {
                name: "users".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                triggers: vec![],
                ddl: None,
            },
        ];
        let tables = vec![
            TableInfo {
                name: "order_items".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            },
            TableInfo {
                name: "orders".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            },
            TableInfo {
                name: "users".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            },
        ];

        let graph = DependencyGraph::build(&details, &tables);
        let drop_order = graph.drop_order();

        let di = drop_order.iter().position(|n| n == "order_items").unwrap();
        let oi = drop_order.iter().position(|n| n == "orders").unwrap();
        assert!(di < oi, "order_items should be dropped before orders");
    }

    #[test]
    fn coverage_score_empty_graph_returns_one() {
        let graph = DependencyGraph { nodes: HashMap::new(), topological_order: vec![] };
        assert_eq!(graph.coverage_score(&[]), 1.0);
    }

    #[test]
    fn coverage_score_partial_coverage() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            DependencyNode { table_name: "a".to_string(), depends_on: vec!["b".to_string()], depended_by: vec![] },
        );
        nodes.insert(
            "b".to_string(),
            DependencyNode {
                table_name: "b".to_string(),
                depends_on: vec!["c".to_string()],
                depended_by: vec!["a".to_string()],
            },
        );
        nodes.insert(
            "c".to_string(),
            DependencyNode { table_name: "c".to_string(), depends_on: vec![], depended_by: vec!["b".to_string()] },
        );
        let graph =
            DependencyGraph { nodes, topological_order: vec!["c".to_string(), "b".to_string(), "a".to_string()] };

        let score = graph.coverage_score(&["a".to_string(), "b".to_string()]);
        assert!((score - 0.5).abs() < 0.01);
    }

    #[test]
    fn coverage_score_level2_transitive_edges() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            DependencyNode { table_name: "a".to_string(), depends_on: vec!["b".to_string()], depended_by: vec![] },
        );
        nodes.insert(
            "b".to_string(),
            DependencyNode {
                table_name: "b".to_string(),
                depends_on: vec!["c".to_string()],
                depended_by: vec!["a".to_string()],
            },
        );
        nodes.insert(
            "c".to_string(),
            DependencyNode { table_name: "c".to_string(), depends_on: vec![], depended_by: vec!["b".to_string()] },
        );
        let graph =
            DependencyGraph { nodes, topological_order: vec!["c".to_string(), "b".to_string(), "a".to_string()] };

        let l2_score = graph.coverage_score_level2(&["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!((l2_score - 1.0).abs() < 0.01, "full coverage should give 1.0");
    }

    #[test]
    fn coverage_score_level2_partial() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            DependencyNode { table_name: "a".to_string(), depends_on: vec!["b".to_string()], depended_by: vec![] },
        );
        nodes.insert(
            "b".to_string(),
            DependencyNode {
                table_name: "b".to_string(),
                depends_on: vec!["c".to_string()],
                depended_by: vec!["a".to_string()],
            },
        );
        nodes.insert(
            "c".to_string(),
            DependencyNode { table_name: "c".to_string(), depends_on: vec![], depended_by: vec!["b".to_string()] },
        );
        let graph =
            DependencyGraph { nodes, topological_order: vec!["c".to_string(), "b".to_string(), "a".to_string()] };

        let l2_score = graph.coverage_score_level2(&["a".to_string(), "b".to_string()]);
        assert!((l2_score - 0.0).abs() < 0.01, "missing grandparent c means 0 transitive coverage");
    }

    #[test]
    fn composite_coverage_full_coverage() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            DependencyNode { table_name: "a".to_string(), depends_on: vec!["b".to_string()], depended_by: vec![] },
        );
        nodes.insert(
            "b".to_string(),
            DependencyNode {
                table_name: "b".to_string(),
                depends_on: vec!["c".to_string()],
                depended_by: vec!["a".to_string()],
            },
        );
        nodes.insert(
            "c".to_string(),
            DependencyNode { table_name: "c".to_string(), depends_on: vec![], depended_by: vec!["b".to_string()] },
        );
        let graph =
            DependencyGraph { nodes, topological_order: vec!["c".to_string(), "b".to_string(), "a".to_string()] };

        let report = graph.composite_coverage_score(&["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!((report.level1_score - 1.0).abs() < 0.01);
        assert!((report.level2_score - 1.0).abs() < 0.01);
        assert!((report.composite_score - 1.0).abs() < 0.01);
        assert_eq!(report.level1_covered, 2);
        assert_eq!(report.level1_total, 2);
        assert_eq!(report.level2_covered, 1);
        assert_eq!(report.level2_total, 1);
    }

    #[test]
    fn composite_coverage_partial() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "a".to_string(),
            DependencyNode { table_name: "a".to_string(), depends_on: vec!["b".to_string()], depended_by: vec![] },
        );
        nodes.insert(
            "b".to_string(),
            DependencyNode {
                table_name: "b".to_string(),
                depends_on: vec!["c".to_string()],
                depended_by: vec!["a".to_string()],
            },
        );
        nodes.insert(
            "c".to_string(),
            DependencyNode { table_name: "c".to_string(), depends_on: vec![], depended_by: vec!["b".to_string()] },
        );
        let graph =
            DependencyGraph { nodes, topological_order: vec!["c".to_string(), "b".to_string(), "a".to_string()] };

        let report = graph.composite_coverage_score(&["a".to_string(), "b".to_string()]);
        assert!((report.level1_score - 0.5).abs() < 0.01);
        assert!((report.level2_score - 0.0).abs() < 0.01);
        assert!((report.composite_score - 0.3).abs() < 0.01, "0.6*0.5 + 0.4*0.0 = 0.3");
        assert_eq!(report.level1_covered, 1);
        assert_eq!(report.level1_total, 2);
        assert!(!report.uncovered_edges.is_empty());
    }

    #[test]
    fn composite_coverage_no_dependencies() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "t1".to_string(),
            DependencyNode { table_name: "t1".to_string(), depends_on: vec![], depended_by: vec![] },
        );
        nodes.insert(
            "t2".to_string(),
            DependencyNode { table_name: "t2".to_string(), depends_on: vec![], depended_by: vec![] },
        );
        let graph = DependencyGraph { nodes, topological_order: vec!["t1".to_string(), "t2".to_string()] };

        let report = graph.composite_coverage_score(&["t1".to_string()]);
        assert!((report.level1_score - 1.0).abs() < 0.01);
        assert!((report.level2_score - 1.0).abs() < 0.01);
        assert!((report.composite_score - 1.0).abs() < 0.01);
        assert!(report.uncovered_edges.is_empty());
    }

    // ========================================================================
    // Phase 4.1: Rename Detection Tests
    // ========================================================================

    #[test]
    fn detect_renames_high_similarity_columns() {
        let source_details = vec![TableSchemaDetail {
            name: "users_old".to_string(),
            columns: vec![
                column("id", "int", None),
                column("name", "varchar(100)", None),
                column("email", "varchar(255)", None),
                column("created_at", "datetime", None),
            ],
            indexes: vec![],
            foreign_keys: vec![],
            triggers: vec![],
            ddl: None,
        }];
        let target_details = vec![TableSchemaDetail {
            name: "users_new".to_string(),
            columns: vec![
                column("id", "integer", None),
                column("name", "varchar(100)", None),
                column("email", "varchar(255)", None),
                column("updated_at", "datetime", None),
            ],
            indexes: vec![],
            foreign_keys: vec![],
            triggers: vec![],
            ddl: None,
        }];

        let candidates = detect_renames(
            &["users_new".to_string()],
            &["users_old".to_string()],
            &source_details,
            &target_details,
            0.5,
        );

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].removed_name, "users_new");
        assert_eq!(candidates[0].added_name, "users_old");
        assert!(candidates[0].score >= 0.5);
    }

    #[test]
    fn detect_renames_low_similarity_below_threshold() {
        let source_details = vec![TableSchemaDetail {
            name: "users".to_string(),
            columns: vec![column("id", "int", None)],
            indexes: vec![],
            foreign_keys: vec![],
            triggers: vec![],
            ddl: None,
        }];
        let target_details = vec![TableSchemaDetail {
            name: "products".to_string(),
            columns: vec![column("sku", "varchar(50)", None), column("price", "decimal", None)],
            indexes: vec![],
            foreign_keys: vec![],
            triggers: vec![],
            ddl: None,
        }];

        let candidates =
            detect_renames(&["users".to_string()], &["products".to_string()], &source_details, &target_details, 0.5);

        assert!(candidates.is_empty());
    }

    #[test]
    fn jaccard_similarity_identical_sets() {
        let a: HashSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    // ========================================================================
    // Phase 4.2: Batch Naming Pattern Tests
    // ========================================================================

    #[test]
    fn batch_pattern_matching_wildcard() {
        let source = vec!["log_2024_01".to_string(), "log_2024_02".to_string(), "users".to_string()];
        let target = vec!["log_2024_03".to_string()];
        let patterns = vec![BatchPattern {
            pattern: "log_*".to_string(),
            is_regex: false,
            description: "all log tables".to_string(),
        }];

        let (added, removed, common, matches) = diff_names_with_patterns(&source, &target, &patterns);
        assert_eq!(removed, vec!["log_2024_03"]);
        assert_eq!(common.len(), 0);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].len(), 2);
    }

    #[test]
    fn batch_pattern_regex_matching() {
        let source = vec!["tbl_001".to_string(), "tbl_002".to_string(), "other".to_string()];
        let target = vec![];
        let patterns = vec![BatchPattern {
            pattern: r"tbl_\d{3}".to_string(),
            is_regex: true,
            description: "numbered tables".to_string(),
        }];

        let (_added, _removed, _common, matches) = diff_names_with_patterns(&source, &target, &patterns);
        assert_eq!(matches[0].len(), 2);
    }

    #[test]
    fn pattern_conflict_detection() {
        let patterns = vec![
            BatchPattern { pattern: "user_*".to_string(), is_regex: false, description: "user tables".to_string() },
            BatchPattern {
                pattern: "user_data".to_string(),
                is_regex: false,
                description: "specific user data".to_string(),
            },
        ];

        let names = vec!["user_data".to_string(), "user_log".to_string()];
        let conflicts = detect_pattern_conflicts(&patterns, &names);
        assert!(!conflicts.is_empty());
    }

    // ========================================================================
    // Phase 4.3: Type Compatibility Tests
    // ========================================================================

    #[test]
    fn diff_columns_with_compatibility_integer_family() {
        let (_diffs, warnings) = diff_columns_with_compatibility(
            &[column("id", "INT", None)],
            &[column("id", "BIGINT", None)],
            false,
            false,
            DialectKind::Mysql,
            DialectKind::Mysql,
            0.9,
        );
        assert!(!warnings.is_empty());
        assert_eq!(warnings[0].risk, ColumnConversionRisk::Low);
    }

    #[test]
    fn diff_columns_with_compatibility_exact_match_no_warning() {
        let (_diffs, warnings) = diff_columns_with_compatibility(
            &[column("id", "INT", None)],
            &[column("id", "INT", None)],
            false,
            false,
            DialectKind::Mysql,
            DialectKind::Mysql,
            0.5,
        );
        assert!(warnings.is_empty());
    }

    // ========================================================================
    // Phase 4.4: Bidirectional Diff & Rollback Tests
    // ========================================================================

    fn make_diff(diff_type: &str, name: &str) -> TableDiff {
        TableDiff {
            diff_type: diff_type.to_string(),
            object_type: Some("table".to_string()),
            name: name.to_string(),
            columns: None,
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        }
    }

    #[test]
    fn rollback_graph_add_becomes_drop() {
        let diffs = vec![make_diff("added", "new_table")];
        let dep_graph = DependencyGraph { nodes: HashMap::new(), topological_order: vec![] };
        let graph = RollbackGraph::from_forward_diffs(&diffs, &[], &dep_graph);

        assert_eq!(graph.forward_nodes.len(), 1);
        assert_eq!(graph.rollback_nodes.len(), 1);
        assert_eq!(graph.forward_nodes[0].table_diff.diff_type, "added");
        assert_eq!(graph.rollback_nodes[0].table_diff.diff_type, "removed");
    }

    #[test]
    fn rollback_graph_remove_becomes_add() {
        let diffs = vec![make_diff("removed", "old_table")];
        let dep_graph = DependencyGraph { nodes: HashMap::new(), topological_order: vec![] };
        let graph = RollbackGraph::from_forward_diffs(&diffs, &[], &dep_graph);

        assert_eq!(graph.rollback_nodes[0].table_diff.diff_type, "added");
        assert_eq!(graph.rollback_nodes[0].table_diff.ddl, None);
    }

    #[test]
    fn rollback_graph_modified_stays_modified_swapped() {
        let source_col = column("name", "varchar(100)", None);
        let target_col = column("name", "varchar(50)", None);
        let diffs = vec![TableDiff {
            diff_type: "modified".to_string(),
            object_type: Some("table".to_string()),
            name: "users".to_string(),
            columns: Some(vec![ColumnDiff {
                diff_type: "modified".to_string(),
                name: "name".to_string(),
                source: Some(source_col.clone()),
                target: Some(target_col.clone()),
                changes: vec!["type: varchar(50) → varchar(100)".to_string()],
            }]),
            indexes: None,
            foreign_keys: None,
            triggers: None,
            ddl: None,
            target_ddl: None,
            source_table_comment: None,
            target_table_comment: None,
            sync_sql: None,
        }];

        let dep_graph = DependencyGraph { nodes: HashMap::new(), topological_order: vec![] };
        let graph = RollbackGraph::from_forward_diffs(&diffs, &[], &dep_graph);

        let rollback = &graph.rollback_nodes[0];
        assert_eq!(rollback.table_diff.diff_type, "modified");
        let rb_cols = rollback.table_diff.columns.as_ref().unwrap();
        assert_eq!(rb_cols[0].diff_type, "modified");
        assert_eq!(rb_cols[0].source.as_ref().unwrap().data_type, "varchar(50)");
        assert_eq!(rb_cols[0].target.as_ref().unwrap().data_type, "varchar(100)");
    }

    #[test]
    fn rollback_consistency_validation() {
        let diffs = vec![make_diff("added", "t1"), make_diff("removed", "t2")];
        let dep_graph = DependencyGraph { nodes: HashMap::new(), topological_order: vec![] };
        let mut graph = RollbackGraph::from_forward_diffs(&diffs, &[], &dep_graph);
        assert!(graph.validate_consistency());
        assert!(graph.consistency_issues.is_empty());
    }

    // ========================================================================
    // Phase 4.6: Permission Tests
    // ========================================================================

    #[test]
    fn diff_permissions_detects_added_and_removed() {
        let source = vec![PermissionInfo {
            grantee: "app_user".to_string(),
            object_type: "TABLE".to_string(),
            object_name: "orders".to_string(),
            privilege: "SELECT".to_string(),
            is_grantable: false,
        }];
        let target = vec![PermissionInfo {
            grantee: "app_user".to_string(),
            object_type: "TABLE".to_string(),
            object_name: "orders".to_string(),
            privilege: "INSERT".to_string(),
            is_grantable: false,
        }];

        let diffs = diff_permissions(&source, &target);
        assert_eq!(diffs.len(), 2);
        assert!(diffs.iter().any(|d| d.diff_type == "added"));
        assert!(diffs.iter().any(|d| d.diff_type == "removed"));
    }

    #[test]
    fn generate_permission_sql_mysql() {
        let diffs = vec![PermissionDiff {
            diff_type: "added".to_string(),
            grantee: "app_user".to_string(),
            object_name: "orders".to_string(),
            privilege: "SELECT".to_string(),
            source: Some(PermissionInfo {
                grantee: "app_user".to_string(),
                object_type: "TABLE".to_string(),
                object_name: "orders".to_string(),
                privilege: "SELECT".to_string(),
                is_grantable: true,
            }),
            target: None,
        }];

        let sql = generate_permission_sync_sql(&diffs, DatabaseType::Mysql, Some("mydb"));
        assert!(sql.contains("GRANT SELECT ON `mydb`.`orders` TO 'app_user' WITH GRANT OPTION"));
    }

    #[test]
    fn generate_permission_sql_postgres_revoke() {
        let diffs = vec![PermissionDiff {
            diff_type: "removed".to_string(),
            grantee: "old_user".to_string(),
            object_name: "users".to_string(),
            privilege: "INSERT".to_string(),
            source: None,
            target: Some(PermissionInfo {
                grantee: "old_user".to_string(),
                object_type: "TABLE".to_string(),
                object_name: "users".to_string(),
                privilege: "INSERT".to_string(),
                is_grantable: false,
            }),
        }];

        let sql = generate_permission_sync_sql(&diffs, DatabaseType::Postgres, Some("public"));
        assert!(sql.contains("REVOKE INSERT ON TABLE \"public\".\"users\" FROM \"old_user\""));
    }

    // ========================================================================
    // Phase 4.7: Resource Scheduling Tests
    // ========================================================================

    #[test]
    fn adaptive_scheduler_optimal_batch_size() {
        let constraint = ResourceConstraint::default();
        let scheduler = AdaptiveScheduler::new(constraint, 400);
        let batch = scheduler.optimal_batch_size();
        assert!(batch > 0);
        assert!(batch <= 50);
    }

    #[test]
    fn adaptive_scheduler_shard_count() {
        let constraint = ResourceConstraint::default();
        let scheduler = AdaptiveScheduler::new(constraint, 200);
        let count = scheduler.recommended_shard_count();
        assert!(count >= 1);
        assert!(count <= 4);
    }

    // ========================================================================
    // Phase 4.8: Backward Compatibility Tests
    // ========================================================================

    #[test]
    fn new_options_default_values_do_not_affect_basic_diff() {
        let options = SchemaDiffPreparationOptions::default();
        let result = prepare_schema_diff(options);
        assert!(result.diffs.is_empty());
        assert!(result.sync_sql.is_empty());
        assert!(result.rollback_sync_sql.is_none());
        assert!(result.rename_candidates.is_empty());
        assert!(result.rollback_graph.is_none());
        assert!(result.compatibility_warnings.is_empty());
        assert!(result.permission_diffs.is_empty());
    }

    #[test]
    fn prepare_schema_diff_with_rename_detection() {
        let options = SchemaDiffPreparationOptions {
            source_tables: vec![TableInfo {
                name: "users_old".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            }],
            target_tables: vec![TableInfo {
                name: "users_new".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            }],
            source_details: vec![TableSchemaDetail {
                name: "users_old".to_string(),
                columns: vec![column("id", "int", None), column("name", "varchar(100)", None)],
                indexes: vec![],
                foreign_keys: vec![],
                triggers: vec![],
                ddl: None,
            }],
            target_details: vec![TableSchemaDetail {
                name: "users_new".to_string(),
                columns: vec![column("id", "int", None), column("name", "varchar(100)", None)],
                indexes: vec![],
                foreign_keys: vec![],
                triggers: vec![],
                ddl: None,
            }],
            source_functions: vec![],
            target_functions: vec![],
            source_sequences: vec![],
            target_sequences: vec![],
            source_rules: vec![],
            target_rules: vec![],
            source_owners: vec![],
            target_owners: vec![],
            database_type: DatabaseType::Mysql,
            target_schema: None,
            ignore_comments: false,
            cascade_delete: false,
            compare_column_order: false,
            detect_renames: true,
            rename_threshold: 0.5,
            ..Default::default()
        };

        let result = prepare_schema_diff(options);
        assert!(!result.rename_candidates.is_empty());
        assert!(result.rename_candidates[0].score >= 0.5);
    }

    #[test]
    fn prepare_schema_diff_with_rollback_generates_rollback_sql() {
        let options = SchemaDiffPreparationOptions {
            source_tables: vec![TableInfo {
                name: "new_table".to_string(),
                table_type: "BASE TABLE".to_string(),
                comment: None,
                parent_schema: None,
                parent_name: None,
            }],
            target_tables: vec![],
            source_details: vec![TableSchemaDetail {
                name: "new_table".to_string(),
                columns: vec![column("id", "int", None)],
                indexes: vec![],
                foreign_keys: vec![],
                triggers: vec![],
                ddl: Some("CREATE TABLE new_table (id int);".to_string()),
            }],
            target_details: vec![],
            source_functions: vec![],
            target_functions: vec![],
            source_sequences: vec![],
            target_sequences: vec![],
            source_rules: vec![],
            target_rules: vec![],
            source_owners: vec![],
            target_owners: vec![],
            database_type: DatabaseType::Mysql,
            target_schema: None,
            ignore_comments: false,
            cascade_delete: false,
            compare_column_order: false,
            enable_rollback: true,
            ..Default::default()
        };

        let result = prepare_schema_diff(options);
        assert!(result.rollback_sync_sql.is_some());
        assert!(result.rollback_graph.is_some());
        let graph = result.rollback_graph.unwrap();
        assert!(graph.is_consistent);
        assert_eq!(graph.forward_nodes.len(), 1);
        assert_eq!(graph.rollback_nodes.len(), 1);
        assert_eq!(graph.rollback_nodes[0].table_diff.diff_type, "removed");
    }
}
