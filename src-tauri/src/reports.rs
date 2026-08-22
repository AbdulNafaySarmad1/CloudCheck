use serde::{Deserialize, Serialize};

use crate::{
    domain::{Scan, Severity},
    error::{AppError, AppResult},
};

const MAX_REPORT_BYTES: usize = 100 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Html,
    Csv,
    Sarif,
    Pdf,
}

impl ReportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Html => "html",
            Self::Csv => "csv",
            Self::Sarif => "sarif",
            Self::Pdf => "pdf",
        }
    }
}

pub fn render(scan: &Scan, format: ReportFormat) -> AppResult<Vec<u8>> {
    let report = match format {
        ReportFormat::Json => serde_json::to_vec_pretty(scan).map_err(|_| AppError::Internal),
        ReportFormat::Html => Ok(render_html(scan).into_bytes()),
        ReportFormat::Csv => Ok(render_csv(scan).into_bytes()),
        ReportFormat::Sarif => render_sarif(scan),
        ReportFormat::Pdf => Ok(render_pdf(scan)),
    }?;
    if report.len() > MAX_REPORT_BYTES {
        return Err(AppError::FileTooLarge);
    }
    Ok(report)
}

fn render_html(scan: &Scan) -> String {
    let mut rows = String::new();
    for finding in &scan.findings {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
            escape_html(&format!("{:?}", finding.severity).to_lowercase()),
            escape_html(&finding.title),
            escape_html(&finding.resource_id),
            escape_html(&finding.evidence),
            escape_html(&finding.remediation),
        ));
    }
    let limitations = scan
        .limitations
        .iter()
        .map(|item| format!("<li>{}</li>", escape_html(item)))
        .collect::<String>();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>Nocturne CloudCheck report</title><style>body{{font:14px system-ui,sans-serif;color:#18202b;margin:32px}}table{{border-collapse:collapse;width:100%}}th,td{{border:1px solid #ccd3dc;padding:8px;text-align:left;vertical-align:top}}th{{background:#eef2f6}}code{{overflow-wrap:anywhere}}.meta{{display:grid;grid-template-columns:max-content 1fr;gap:6px 16px}}</style></head><body><h1>Nocturne CloudCheck</h1><div class=\"meta\"><strong>Product version</strong><span>{}</span><strong>Ruleset version</strong><span>{}</span><strong>Scan ID</strong><span>{}</span><strong>Provider/account</strong><span>AWS / {}</span><strong>Scope</strong><span>{}</span><strong>Scan time</strong><span>{}</span><strong>Resources</strong><span>{}</span></div><h2>Findings ({})</h2><table><thead><tr><th>Severity</th><th>Finding</th><th>Resource</th><th>Evidence</th><th>Remediation</th></tr></thead><tbody>{}</tbody></table><h2>Limitations</h2><ul>{}</ul></body></html>",
        escape_html(&scan.product_version),
        escape_html(&scan.ruleset_version),
        escape_html(&scan.id),
        escape_html(&scan.account_id),
        escape_html(&scan.scope.join(", ")),
        escape_html(&scan.scanned_at.to_rfc3339()),
        scan.resource_count,
        scan.findings.len(),
        rows,
        limitations,
    )
}

fn render_csv(scan: &Scan) -> String {
    let mut output = String::from(
        "scan_id,scanned_at,product_version,ruleset_version,account_id,scope,rule_id,severity,title,resource_id,region,evidence,remediation\r\n",
    );
    for finding in &scan.findings {
        let scanned_at = scan.scanned_at.to_rfc3339();
        let scope = scan.scope.join(";");
        let severity = format!("{:?}", finding.severity).to_lowercase();
        let fields = [
            scan.id.as_str(),
            scanned_at.as_str(),
            scan.product_version.as_str(),
            scan.ruleset_version.as_str(),
            scan.account_id.as_str(),
            scope.as_str(),
            finding.rule_id.as_str(),
            severity.as_str(),
            finding.title.as_str(),
            finding.resource_id.as_str(),
            finding.region.as_deref().unwrap_or(""),
            finding.evidence.as_str(),
            finding.remediation.as_str(),
        ];
        output.push_str(&fields.map(csv_field).join(","));
        output.push_str("\r\n");
    }
    output
}

fn render_sarif(scan: &Scan) -> AppResult<Vec<u8>> {
    let results: Vec<_> = scan
        .findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "ruleId": finding.rule_id,
                "level": sarif_level(finding.severity),
                "message": { "text": finding.title },
                "fingerprints": { "nocturne/v1": finding.fingerprint },
                "properties": {
                    "resourceId": finding.resource_id,
                    "region": finding.region,
                    "evidence": finding.evidence,
                    "remediation": finding.remediation,
                    "scanId": scan.id
                }
            })
        })
        .collect();
    let report = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "Nocturne CloudCheck",
                "semanticVersion": scan.product_version,
                "informationUri": "https://nocturne.invalid/cloudcheck",
                "properties": { "rulesetVersion": scan.ruleset_version }
            }},
            "invocations": [{
                "executionSuccessful": true,
                "endTimeUtc": scan.scanned_at.to_rfc3339(),
                "properties": {
                    "scope": scan.scope,
                    "accountId": scan.account_id,
                    "limitations": scan.limitations
                }
            }],
            "results": results
        }]
    });
    serde_json::to_vec_pretty(&report).map_err(|_| AppError::Internal)
}

fn render_pdf(scan: &Scan) -> Vec<u8> {
    let mut lines = vec![
        "Nocturne CloudCheck Security Report".to_owned(),
        format!("Scan ID: {}", scan.id),
        format!("Time: {}", scan.scanned_at.to_rfc3339()),
        format!(
            "Product: {}  Rules: {}",
            scan.product_version, scan.ruleset_version
        ),
        format!(
            "AWS account: {}  Scope: {}",
            scan.account_id,
            scan.scope.join(", ")
        ),
        format!(
            "Resources: {}  Findings: {}",
            scan.resource_count,
            scan.findings.len()
        ),
        String::new(),
    ];
    for finding in &scan.findings {
        lines.push(format!(
            "[{:?}] {} - {}",
            finding.severity, finding.rule_id, finding.title
        ));
        lines.push(format!("Resource: {}", finding.resource_id));
        lines.push(format!("Remediation: {}", finding.remediation));
        lines.push(String::new());
    }
    lines.push("Limitations".into());
    lines.extend(scan.limitations.iter().map(|item| format!("- {item}")));

    let pages: Vec<_> = lines.chunks(52).collect();
    let mut objects: Vec<(usize, Vec<u8>)> = Vec::new();
    objects.push((1, b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()));
    let kids = (0..pages.len())
        .map(|index| format!("{} 0 R", 4 + index * 2))
        .collect::<Vec<_>>()
        .join(" ");
    objects.push((
        2,
        format!("<< /Type /Pages /Kids [{kids}] /Count {} >>", pages.len()).into_bytes(),
    ));
    objects.push((
        3,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    ));
    for (index, page_lines) in pages.iter().enumerate() {
        let page_id = 4 + index * 2;
        let stream_id = page_id + 1;
        objects.push((page_id, format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents {stream_id} 0 R >>").into_bytes()));
        let mut commands = String::from("BT /F1 9 Tf 36 756 Td 13 TL ");
        for line in *page_lines {
            commands.push('(');
            commands.push_str(&pdf_text(line));
            commands.push_str(") Tj T* ");
        }
        commands.push_str("ET");
        objects.push((
            stream_id,
            format!(
                "<< /Length {} >>\nstream\n{}\nendstream",
                commands.len(),
                commands
            )
            .into_bytes(),
        ));
    }

    let mut pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let object_count = objects.len();
    let mut offsets = vec![0usize; object_count + 1];
    for (id, object) in objects {
        offsets[id] = pdf.len();
        pdf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        pdf.extend_from_slice(&object);
        pdf.extend_from_slice(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", object_count + 1).as_bytes(),
    );
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            object_count + 1
        )
        .as_bytes(),
    );
    pdf
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn csv_field(value: &str) -> String {
    // Leading formula characters are neutralized for spreadsheet consumers.
    let safe = if value
        .trim_start_matches([' ', '\t', '\r', '\n'])
        .chars()
        .next()
        .is_some_and(|character| matches!(character, '=' | '+' | '-' | '@'))
    {
        format!("'{value}")
    } else {
        value.to_owned()
    };
    format!("\"{}\"", safe.replace('"', "\"\""))
}

fn pdf_text(value: &str) -> String {
    value
        .chars()
        .take(180)
        .map(|character| {
            if character.is_ascii_graphic() || character == ' ' {
                character
            } else {
                '?'
            }
        })
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical | Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low | Severity::Info => "note",
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::{Finding, Provider};

    fn scan() -> Scan {
        Scan {
            id: "scan".into(),
            provider: Provider::Aws,
            account_id: "<script>alert(1)</script>".into(),
            scope: vec!["us-east-1".into()],
            scanned_at: Utc::now(),
            product_version: "1".into(),
            ruleset_version: "1".into(),
            resource_count: 1,
            findings: vec![Finding {
                fingerprint: "fingerprint".into(),
                rule_id: "RULE".into(),
                title: "=HYPERLINK(\"bad\")".into(),
                severity: Severity::High,
                resource_id: "bucket<script>".into(),
                region: None,
                evidence: "evidence".into(),
                remediation: "fix it".into(),
            }],
            limitations: vec!["limited".into()],
        }
    }

    #[test]
    fn html_escapes_untrusted_values() {
        let html = String::from_utf8(render(&scan(), ReportFormat::Html).unwrap()).unwrap();
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn csv_neutralizes_formulas() {
        let mut input = scan();
        input.scope = vec!["\t=HYPERLINK(\"bad\")".into()];
        let csv = String::from_utf8(render(&input, ReportFormat::Csv).unwrap()).unwrap();
        assert!(csv.contains("\"'=HYPERLINK"));
        assert!(csv.contains("\"'\t=HYPERLINK"));
    }

    #[test]
    fn pdf_has_valid_markers() {
        let pdf = render(&scan(), ReportFormat::Pdf).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
    }
}
