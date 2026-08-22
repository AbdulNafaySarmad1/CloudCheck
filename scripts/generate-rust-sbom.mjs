import { mkdir, readFile, unlink, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";

const result = spawnSync("cargo", ["cyclonedx", "--all", "--format", "json", "--override-filename", "nocturne-cloudcheck-rust.cdx"], {
  encoding: "utf8",
  stdio: "inherit",
});
if (result.status !== 0) process.exit(result.status ?? 1);

await mkdir("artifacts/sbom", { recursive: true });
const source = "src-tauri/nocturne-cloudcheck-rust.cdx.json";
const sbom = sanitize(JSON.parse(await readFile(source, "utf8")));
await writeFile("artifacts/sbom/nocturne-cloudcheck-rust.cdx.json", `${JSON.stringify(sbom, null, 2)}\n`);
await unlink(source);
console.log("Wrote artifacts/sbom/nocturne-cloudcheck-rust.cdx.json");

function sanitize(value) {
  if (Array.isArray(value)) return value.map(sanitize);
  if (value && typeof value === "object") return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, sanitize(item)]));
  if (typeof value !== "string") return value;
  const localReference = value.match(/^path\+file:\/\/.*#(.+)$/);
  if (localReference) return `pkg:cargo/${localReference[1]}`;
  return value.replace(/\?download_url=file:\/\/\./, "");
}
