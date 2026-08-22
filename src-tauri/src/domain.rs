use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RULESET_VERSION: &str = "2026.08.1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: u32,
    pub provider: Provider,
    pub account_id: String,
    #[serde(default)]
    pub scope: Vec<String>,
    pub resources: Vec<Resource>,
}

impl Snapshot {
    pub fn validate(&self) -> AppResult<()> {
        if self.schema_version != 1 {
            return Err(AppError::InvalidInput("unsupported schema version".into()));
        }
        if self.provider != Provider::Aws {
            return Err(AppError::InvalidInput(
                "only AWS is currently supported".into(),
            ));
        }
        validate_identifier(&self.account_id, 64)?;
        if self.scope.len() > 128 || self.resources.len() > 10_000 {
            return Err(AppError::InvalidInput(
                "snapshot collection is too large".into(),
            ));
        }
        for scope in &self.scope {
            validate_identifier(scope, 128)?;
        }
        let mut resource_ids = BTreeSet::new();
        for resource in &self.resources {
            resource.validate()?;
            if !resource_ids.insert(resource.id.as_str()) {
                return Err(AppError::InvalidInput("duplicate resource id".into()));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Aws,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub id: String,
    pub kind: ResourceKind,
    pub region: Option<String>,
    #[serde(default)]
    pub properties: BTreeMap<String, serde_json::Value>,
}

impl Resource {
    fn validate(&self) -> AppResult<()> {
        validate_identifier(&self.id, 512)?;
        if let Some(region) = &self.region {
            validate_identifier(region, 64)?;
        }
        if self.properties.len() > 128 {
            return Err(AppError::InvalidInput(
                "too many resource properties".into(),
            ));
        }
        for (key, value) in &self.properties {
            validate_identifier(key, 128)?;
            validate_json_value(value, 0)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    S3Bucket,
    IamPolicy,
    SecurityGroup,
    CloudTrail,
    Account,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub fingerprint: String,
    pub rule_id: String,
    pub title: String,
    pub severity: Severity,
    pub resource_id: String,
    pub region: Option<String>,
    pub evidence: String,
    pub remediation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    pub id: String,
    pub provider: Provider,
    pub account_id: String,
    pub scope: Vec<String>,
    pub scanned_at: DateTime<Utc>,
    pub product_version: String,
    pub ruleset_version: String,
    pub resource_count: usize,
    pub findings: Vec<Finding>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub id: String,
    pub provider: Provider,
    pub account_id: String,
    pub scanned_at: DateTime<Utc>,
    pub resource_count: usize,
    pub finding_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDiff {
    pub base_scan_id: String,
    pub target_scan_id: String,
    pub added: Vec<Finding>,
    pub resolved: Vec<Finding>,
    pub unchanged_count: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventorySummary {
    pub provider: Provider,
    pub account_id: String,
    pub scope: Vec<String>,
    pub resource_count: usize,
    pub counts_by_kind: BTreeMap<String, usize>,
    pub counts_by_region: BTreeMap<String, usize>,
}

impl From<&Snapshot> for InventorySummary {
    fn from(snapshot: &Snapshot) -> Self {
        let mut counts_by_kind = BTreeMap::new();
        let mut counts_by_region = BTreeMap::new();
        for resource in &snapshot.resources {
            *counts_by_kind
                .entry(format!("{:?}", resource.kind).to_lowercase())
                .or_insert(0) += 1;
            *counts_by_region
                .entry(resource.region.clone().unwrap_or_else(|| "global".into()))
                .or_insert(0) += 1;
        }
        Self {
            provider: snapshot.provider,
            account_id: snapshot.account_id.clone(),
            scope: snapshot.scope.clone(),
            resource_count: snapshot.resources.len(),
            counts_by_kind,
            counts_by_region,
        }
    }
}

fn validate_identifier(value: &str, max: usize) -> AppResult<()> {
    validate_text(value, max)?;
    if value.chars().any(char::is_control) {
        return Err(AppError::InvalidInput("control character".into()));
    }
    Ok(())
}

fn validate_text(value: &str, max: usize) -> AppResult<()> {
    let len = value.chars().count();
    if value.trim().is_empty() || len > max {
        return Err(AppError::InvalidInput("invalid text length".into()));
    }
    Ok(())
}

fn validate_json_value(value: &serde_json::Value, depth: usize) -> AppResult<()> {
    if depth > 8 {
        return Err(AppError::InvalidInput(
            "property nesting is too deep".into(),
        ));
    }
    match value {
        serde_json::Value::String(value) if value.len() > 16_384 => {
            Err(AppError::InvalidInput("property value is too large".into()))
        }
        serde_json::Value::Array(values) if values.len() > 10_000 => {
            Err(AppError::InvalidInput("property array is too large".into()))
        }
        serde_json::Value::Array(values) => values
            .iter()
            .try_for_each(|value| validate_json_value(value, depth + 1)),
        serde_json::Value::Object(values) if values.len() > 256 => Err(AppError::InvalidInput(
            "property object is too large".into(),
        )),
        serde_json::Value::Object(values) => values
            .values()
            .try_for_each(|value| validate_json_value(value, depth + 1)),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_deep_properties() {
        let mut value = serde_json::json!(true);
        for _ in 0..10 {
            value = serde_json::json!({"nested": value});
        }
        assert!(validate_json_value(&value, 0).is_err());
    }

    #[test]
    fn rejects_duplicate_resources_and_control_characters() {
        let snapshot: Snapshot = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "provider": "aws",
            "account_id": "123",
            "scope": ["\t=FORMULA"],
            "resources": []
        }))
        .unwrap();
        assert!(snapshot.validate().is_err());

        let duplicate: Snapshot = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "provider": "aws",
            "account_id": "123",
            "scope": [],
            "resources": [
                {"id":"same", "kind":"account", "region":null, "properties":{}},
                {"id":"same", "kind":"account", "region":null, "properties":{}}
            ]
        }))
        .unwrap();
        assert!(duplicate.validate().is_err());
    }
}
