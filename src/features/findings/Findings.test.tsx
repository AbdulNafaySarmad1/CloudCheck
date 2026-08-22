import { fireEvent, render, screen } from "@testing-library/react";
import { FindingsView } from "./Findings";
import type { Scan } from "../../types";

const scan: Scan = {
  id: "scan-1", provider: "aws", accountId: "123456789012", scope: ["us-east-1"],
  scannedAt: "2026-08-22T10:00:00Z", productVersion: "0.1.0", rulesetVersion: "2026.08.1",
  resourceCount: 1, limitations: [], findings: [{
    fingerprint: "fingerprint", ruleId: "AWS-S3-001", title: "<script>alert(1)</script>",
    severity: "critical", resourceId: "arn:aws:s3:::unsafe", region: "us-east-1",
    evidence: "Untrusted <img src=x> evidence", remediation: "Block public access.",
  }],
};

describe("FindingsView", () => {
  it("renders untrusted finding text without creating markup", () => {
    const { container } = render(<FindingsView scan={scan} onStart={() => undefined} />);
    expect(screen.getByText("<script>alert(1)</script>")).toBeVisible();
    expect(container.querySelector("script")).toBeNull();
    fireEvent.click(screen.getByText("<script>alert(1)</script>"));
    expect(screen.getByRole("dialog", { name: "<script>alert(1)</script>" })).toBeVisible();
    expect(screen.getByText("Untrusted <img src=x> evidence")).toBeVisible();
  });
});
