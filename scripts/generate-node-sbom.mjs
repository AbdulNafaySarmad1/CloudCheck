import { mkdir, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

await mkdir("artifacts/sbom", { recursive: true });
const result = spawnSync("npm", ["sbom", "--package-lock-only", "--omit=dev", "--sbom-format", "cyclonedx", "--sbom-type", "application"], {
  encoding: "utf8",
  maxBuffer: 20 * 1024 * 1024,
});
if (result.status !== 0) {
  process.stderr.write(result.stderr || "npm SBOM generation failed.\n");
  process.exit(result.status ?? 1);
}
const sbom = JSON.parse(result.stdout);
await writeFile("artifacts/sbom/nocturne-cloudcheck-node.cdx.json", `${JSON.stringify(sbom, null, 2)}\n`);
console.log("Wrote artifacts/sbom/nocturne-cloudcheck-node.cdx.json");
