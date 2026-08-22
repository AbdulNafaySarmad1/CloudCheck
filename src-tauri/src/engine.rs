use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::domain::{
    Finding, Resource, ResourceKind, Scan, ScanDiff, Severity, Snapshot, PRODUCT_VERSION,
    RULESET_VERSION,
};

pub fn evaluate(snapshot: &Snapshot, scan_id: String, scanned_at: DateTime<Utc>) -> Scan {
    let mut findings = Vec::new();
    for resource in &snapshot.resources {
        match resource.kind {
            ResourceKind::S3Bucket => evaluate_s3(resource, &mut findings),
            ResourceKind::IamPolicy => evaluate_iam(resource, &mut findings),
            ResourceKind::SecurityGroup => evaluate_security_group(resource, &mut findings),
            ResourceKind::Account => evaluate_account(resource, &mut findings),
            ResourceKind::CloudTrail => evaluate_cloudtrail(resource, &mut findings),
        }
    }
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.rule_id.cmp(&b.rule_id))
            .then_with(|| a.resource_id.cmp(&b.resource_id))
    });

    Scan {
        id: scan_id,
        provider: snapshot.provider,
        account_id: snapshot.account_id.clone(),
        scope: snapshot.scope.clone(),
        scanned_at,
        product_version: PRODUCT_VERSION.into(),
        ruleset_version: RULESET_VERSION.into(),
        resource_count: snapshot.resources.len(),
        findings,
        limitations: vec![
            "Results reflect only resources and properties present in the supplied scope.".into(),
            "CloudCheck does not validate runtime behavior or exploitability.".into(),
        ],
    }
}

pub fn diff(base: &Scan, target: &Scan) -> ScanDiff {
    let base_by_id: BTreeMap<_, _> = base
        .findings
        .iter()
        .map(|finding| (finding.fingerprint.as_str(), finding))
        .collect();
    let target_by_id: BTreeMap<_, _> = target
        .findings
        .iter()
        .map(|finding| (finding.fingerprint.as_str(), finding))
        .collect();
    let base_ids: BTreeSet<_> = base_by_id.keys().copied().collect();
    let target_ids: BTreeSet<_> = target_by_id.keys().copied().collect();

    ScanDiff {
        base_scan_id: base.id.clone(),
        target_scan_id: target.id.clone(),
        added: target_ids
            .difference(&base_ids)
            .filter_map(|id| target_by_id.get(id).copied().cloned())
            .collect(),
        resolved: base_ids
            .difference(&target_ids)
            .filter_map(|id| base_by_id.get(id).copied().cloned())
            .collect(),
        unchanged_count: base_ids.intersection(&target_ids).count(),
    }
}

fn evaluate_s3(resource: &Resource, findings: &mut Vec<Finding>) {
    if bool_property(resource, "public") == Some(true) {
        findings.push(finding(
            "AWS-S3-001",
            "S3 bucket permits public access",
            Severity::Critical,
            resource,
            "The normalized public-access flag is enabled.",
            "Enable all S3 Block Public Access controls and remove public bucket or object ACLs and policies.",
        ));
    }
    if bool_property(resource, "encryption_enabled") == Some(false) {
        findings.push(finding(
            "AWS-S3-002",
            "S3 bucket encryption is disabled",
            Severity::High,
            resource,
            "Default bucket encryption is disabled.",
            "Enable default SSE-S3 or SSE-KMS encryption and constrain KMS key access.",
        ));
    }
}

fn evaluate_iam(resource: &Resource, findings: &mut Vec<Finding>) {
    let wildcard_actions = string_array(resource, "actions").is_some_and(|v| v.contains(&"*"));
    let wildcard_resources = string_array(resource, "resources").is_some_and(|v| v.contains(&"*"));
    if wildcard_actions && wildcard_resources {
        findings.push(finding(
            "AWS-IAM-001",
            "IAM policy grants unrestricted access",
            Severity::Critical,
            resource,
            "The policy contains wildcard actions and wildcard resources.",
            "Replace wildcards with the minimum required actions and resource ARNs; add restrictive conditions.",
        ));
    }
}

fn evaluate_security_group(resource: &Resource, findings: &mut Vec<Finding>) {
    let public = string_array(resource, "cidrs")
        .is_some_and(|v| v.iter().any(|x| *x == "0.0.0.0/0" || *x == "::/0"));
    let administrative_port = resource
        .properties
        .get("ports")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|ports| {
            ports
                .iter()
                .filter_map(serde_json::Value::as_u64)
                .any(|port| port == 22 || port == 3389)
        });
    if public && administrative_port {
        findings.push(finding(
            "AWS-EC2-001",
            "Administrative port is open to the internet",
            Severity::High,
            resource,
            "A normalized ingress rule exposes SSH or RDP to a public CIDR.",
            "Restrict ingress to approved private networks or use AWS Systems Manager Session Manager.",
        ));
    }
}

fn evaluate_account(resource: &Resource, findings: &mut Vec<Finding>) {
    if bool_property(resource, "root_mfa_enabled") == Some(false) {
        findings.push(finding(
            "AWS-IAM-002",
            "Root account MFA is disabled",
            Severity::Critical,
            resource,
            "The account root MFA flag is disabled.",
            "Enroll the root user with a hardware-backed MFA device and avoid routine root use.",
        ));
    }
}

fn evaluate_cloudtrail(resource: &Resource, findings: &mut Vec<Finding>) {
    if bool_property(resource, "multi_region_enabled") == Some(false) {
        findings.push(finding(
            "AWS-CT-001",
            "CloudTrail is not multi-region",
            Severity::High,
            resource,
            "The trail does not cover all current AWS regions.",
            "Configure an organization or multi-region trail with log-file validation and protected storage.",
        ));
    }
}

fn finding(
    rule_id: &str,
    title: &str,
    severity: Severity,
    resource: &Resource,
    evidence: &str,
    remediation: &str,
) -> Finding {
    let mut hash = Sha256::new();
    hash.update(rule_id.as_bytes());
    hash.update([0]);
    hash.update(resource.id.as_bytes());
    let fingerprint = format!("{:x}", hash.finalize());
    Finding {
        fingerprint,
        rule_id: rule_id.into(),
        title: title.into(),
        severity,
        resource_id: resource.id.clone(),
        region: resource.region.clone(),
        evidence: evidence.into(),
        remediation: remediation.into(),
    }
}

fn bool_property(resource: &Resource, key: &str) -> Option<bool> {
    resource
        .properties
        .get(key)
        .and_then(serde_json::Value::as_bool)
}

fn string_array<'a>(resource: &'a Resource, key: &str) -> Option<Vec<&'a str>> {
    Some(
        resource
            .properties
            .get(key)?
            .as_array()?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::TimeZone;

    use super::*;
    use crate::domain::Provider;

    fn snapshot(public: bool) -> Snapshot {
        Snapshot {
            schema_version: 1,
            provider: Provider::Aws,
            account_id: "123456789012".into(),
            scope: vec!["us-east-1".into()],
            resources: vec![Resource {
                id: "arn:aws:s3:::example".into(),
                kind: ResourceKind::S3Bucket,
                region: Some("us-east-1".into()),
                properties: BTreeMap::from([("public".into(), serde_json::json!(public))]),
            }],
        }
    }

    #[test]
    fn evaluation_is_deterministic() {
        let at = Utc.with_ymd_and_hms(2026, 8, 22, 12, 0, 0).unwrap();
        let a = evaluate(&snapshot(true), "scan".into(), at);
        let b = evaluate(&snapshot(true), "scan".into(), at);
        assert_eq!(
            serde_json::to_vec(&a).unwrap(),
            serde_json::to_vec(&b).unwrap()
        );
        assert_eq!(a.findings[0].rule_id, "AWS-S3-001");
    }

    #[test]
    fn diff_tracks_resolved_findings() {
        let at = Utc::now();
        let base = evaluate(&snapshot(true), "base".into(), at);
        let target = evaluate(&snapshot(false), "target".into(), at);
        let result = diff(&base, &target);
        assert_eq!(result.resolved.len(), 1);
        assert!(result.added.is_empty());
    }
}
