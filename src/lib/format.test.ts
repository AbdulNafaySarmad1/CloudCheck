import { defaultExportName, fileNameFromPath, formatDate, shortId } from "./format";

describe("format helpers", () => {
  it("handles cross-platform paths without persisting them", () => {
    expect(fileNameFromPath("C:\\snapshots\\aws.json")).toBe("aws.json");
    expect(fileNameFromPath("/var/data/aws.json")).toBe("aws.json");
  });

  it("creates backend-safe report filenames", () => {
    expect(defaultExportName({ accountId: "123/unsafe", scannedAt: "2026-08-22T10:00:00Z" }, "sarif"))
      .toBe("cloudcheck-123-unsafe-2026-08-22.sarif");
  });

  it("handles malformed dates and shortens long identifiers", () => {
    expect(formatDate("not-a-date")).toBe("Unknown");
    expect(shortId("123456789012345678901234")).toBe("12345678...901234");
  });
});
