import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";

const artifactDirectory = process.env.RELEASE_ARTIFACT_DIR ?? "artifacts/release";
const output = process.env.RELEASE_MANIFEST ?? join(artifactDirectory, "release-manifest.json");
const platform = required("RELEASE_PLATFORM");
const architecture = required("RELEASE_ARCH");
const signingState = required("RELEASE_SIGNING_STATE");
const version = process.env.RELEASE_VERSION ?? JSON.parse(await readFile("package.json", "utf8")).version;
const distributable = /(?:\.app\.tar\.gz|\.(?:deb|rpm|msi|exe|dmg))$/i;

const files = (await walk(artifactDirectory)).filter((path) => distributable.test(path)).sort();
if (files.length === 0) throw new Error(`No distributable artifacts found in ${artifactDirectory}.`);

const artifacts = [];
const checksumLines = [];
for (const path of files) {
  const bytes = await readFile(path);
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  const filename = basename(path);
  artifacts.push({ application: "Nocturne CloudCheck", version, platform, architecture, filename, sha256, signingState });
  checksumLines.push(`${sha256}  ${filename}`);
}

await writeFile(output, `${JSON.stringify({ application: "Nocturne CloudCheck", version, artifacts }, null, 2)}\n`);
await writeFile(join(artifactDirectory, "SHA256SUMS"), `${checksumLines.join("\n")}\n`);
console.log(`Recorded ${artifacts.length} artifact(s) in ${output}.`);

function required(name) {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required.`);
  return value;
}

async function walk(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const paths = await Promise.all(entries.map((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  }));
  return paths.flat();
}
