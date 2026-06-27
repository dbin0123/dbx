use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Controls how `TagInheritanceWhitelist` handles unlisted tags.
///
/// - `Allow` — all tags pass through by default; the whitelist is purely informational.
/// - `Block` — only tags whose keys appear in `allowed_keys` pass through; unlisted tags
///   are collected in `TagValidationResult::blocked` and count as blocking violations.
/// - `Strict` — same key filtering as `Block`, but additionally checks that tag *values*
///   match the base-layer values. Any value mismatch (or blocked key) is reported as a
///   violation, and **all** violations (including value mismatches) count as blocking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagPolicy {
    Allow,
    Block,
    Strict,
}

impl Default for TagPolicy {
    fn default() -> Self {
        TagPolicy::Allow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessTag {
    pub key: String,
    pub value: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub immutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInheritanceWhitelist {
    #[serde(default)]
    pub allowed_keys: HashSet<String>,

    #[serde(default = "default_policy")]
    pub default_policy: TagPolicy,
}

fn default_policy() -> TagPolicy {
    TagPolicy::Block
}

impl Default for TagInheritanceWhitelist {
    fn default() -> Self {
        Self { allowed_keys: HashSet::new(), default_policy: TagPolicy::Block }
    }
}

impl TagInheritanceWhitelist {
    pub fn new(allowed: Vec<String>, policy: TagPolicy) -> Self {
        Self { allowed_keys: allowed.into_iter().collect(), default_policy: policy }
    }

    pub fn is_allowed(&self, key: &str) -> bool {
        match self.default_policy {
            TagPolicy::Allow => true,
            TagPolicy::Block => self.allowed_keys.contains(key),
            TagPolicy::Strict => self.allowed_keys.contains(key),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagValidationResult {
    pub allowed: Vec<BusinessTag>,
    pub blocked: Vec<BusinessTag>,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagValidator {
    pub whitelist: TagInheritanceWhitelist,
    pub strict_mode: bool,
}

impl Default for TagValidator {
    fn default() -> Self {
        Self { whitelist: TagInheritanceWhitelist::default(), strict_mode: false }
    }
}

impl TagValidator {
    pub fn new(whitelist: TagInheritanceWhitelist, strict_mode: bool) -> Self {
        Self { whitelist, strict_mode }
    }

    pub fn validate_tags(&self, tags: &[BusinessTag], base_tags: &HashMap<String, String>) -> TagValidationResult {
        let mut allowed = Vec::new();
        let mut blocked = Vec::new();
        let mut violations = Vec::new();

        for tag in tags {
            if !self.whitelist.is_allowed(&tag.key) {
                blocked.push(tag.clone());
                let msg = if self.strict_mode {
                    format!("BLOCKED: Tag '{}'='{}' is not in whitelist (strict mode)", tag.key, tag.value)
                } else {
                    format!("BLOCKED: Tag '{}'='{}' is not in whitelist", tag.key, tag.value)
                };
                violations.push(msg);
                continue;
            }

            if let Some(base_val) = base_tags.get(&tag.key) {
                if tag.value != *base_val && self.strict_mode {
                    violations.push(format!(
                        "STRICT VIOLATION: Tag '{}'='{}' differs from base value '{}'",
                        tag.key, tag.value, base_val
                    ));
                }
            }

            allowed.push(tag.clone());
        }

        TagValidationResult { allowed, blocked, violations }
    }

    pub fn has_blocking_violations(&self, result: &TagValidationResult) -> bool {
        if self.strict_mode {
            !result.violations.is_empty()
        } else {
            !result.blocked.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_policy_allows_all() {
        let whitelist = TagInheritanceWhitelist::new(vec![], TagPolicy::Allow);
        assert!(whitelist.is_allowed("anything"));
    }

    #[test]
    fn test_block_policy_blocks_unlisted() {
        let whitelist = TagInheritanceWhitelist::new(vec!["allowed_key".to_string()], TagPolicy::Block);
        assert!(whitelist.is_allowed("allowed_key"));
        assert!(!whitelist.is_allowed("blocked_key"));
    }

    #[test]
    fn test_strict_policy_blocks_unlisted() {
        let whitelist = TagInheritanceWhitelist::new(vec!["safe".to_string()], TagPolicy::Strict);
        assert!(whitelist.is_allowed("safe"));
        assert!(!whitelist.is_allowed("unsafe"));
    }

    #[test]
    fn test_validator_allows_all_with_allow_policy() {
        let whitelist = TagInheritanceWhitelist::new(vec![], TagPolicy::Allow);
        let validator = TagValidator::new(whitelist, false);

        let tags = vec![BusinessTag {
            key: "env".to_string(),
            value: "prod".to_string(),
            description: String::new(),
            immutable: false,
        }];
        let base = HashMap::new();
        let result = validator.validate_tags(&tags, &base);
        assert_eq!(result.allowed.len(), 1);
        assert_eq!(result.blocked.len(), 0);
    }

    #[test]
    fn test_validator_blocks_unlisted_in_block_mode() {
        let whitelist = TagInheritanceWhitelist::new(vec!["allowed".to_string()], TagPolicy::Block);
        let validator = TagValidator::new(whitelist, false);

        let tags = vec![
            BusinessTag {
                key: "allowed".to_string(),
                value: "yes".to_string(),
                description: String::new(),
                immutable: false,
            },
            BusinessTag {
                key: "blocked".to_string(),
                value: "no".to_string(),
                description: String::new(),
                immutable: false,
            },
        ];
        let base = HashMap::new();
        let result = validator.validate_tags(&tags, &base);
        assert_eq!(result.allowed.len(), 1);
        assert_eq!(result.blocked.len(), 1);
        assert_eq!(result.blocked[0].key, "blocked");
    }

    #[test]
    fn test_strict_mode_reports_value_mismatch() {
        let whitelist = TagInheritanceWhitelist::new(vec!["env".to_string()], TagPolicy::Strict);
        let validator = TagValidator::new(whitelist, true);

        let tags = vec![BusinessTag {
            key: "env".to_string(),
            value: "staging".to_string(),
            description: String::new(),
            immutable: false,
        }];
        let mut base = HashMap::new();
        base.insert("env".to_string(), "prod".to_string());
        let result = validator.validate_tags(&tags, &base);
        assert_eq!(result.violations.len(), 1);
        assert!(result.violations[0].contains("STRICT VIOLATION"));
    }

    #[test]
    fn test_has_blocking_violations_standard_mode() {
        let whitelist = TagInheritanceWhitelist::new(vec![], TagPolicy::Block);
        let validator = TagValidator::new(whitelist, false);

        let result = TagValidationResult {
            allowed: vec![],
            blocked: vec![BusinessTag {
                key: "x".to_string(),
                value: "y".to_string(),
                description: String::new(),
                immutable: false,
            }],
            violations: vec![],
        };
        assert!(validator.has_blocking_violations(&result));
    }

    #[test]
    fn test_has_blocking_violations_strict_mode() {
        let whitelist = TagInheritanceWhitelist::new(vec![], TagPolicy::Strict);
        let validator = TagValidator::new(whitelist, true);

        let result =
            TagValidationResult { allowed: vec![], blocked: vec![], violations: vec!["STRICT VIOLATION".to_string()] };
        assert!(validator.has_blocking_violations(&result));
    }

    #[test]
    fn test_has_blocking_violations_no_violations() {
        let whitelist = TagInheritanceWhitelist::new(vec![], TagPolicy::Allow);
        let validator = TagValidator::new(whitelist, false);

        let result = TagValidationResult { allowed: vec![], blocked: vec![], violations: vec![] };
        assert!(!validator.has_blocking_violations(&result));
    }

    #[test]
    fn test_immutable_tag_field() {
        let tag = BusinessTag {
            key: "env".to_string(),
            value: "prod".to_string(),
            description: "Environment".to_string(),
            immutable: true,
        };
        assert!(tag.immutable);
    }
}
