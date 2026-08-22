import { readFile } from "node:fs/promises";

const rootPackage = JSON.parse(await readFile("package.json", "utf8"));
const packageLock = JSON.parse(await readFile("package-lock.json", "utf8"));
const tauri = JSON.parse(await readFile("src-tauri/tauri.conf.json", "utf8"));
const cargo = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  "package.json": rootPackage.version,
  "package-lock.json": packageLock.version,
  "package-lock root": packageLock.packages?.[""]?.version,
  "src-tauri/Cargo.toml": cargoVersion,
  "src-tauri/tauri.conf.json": tauri.version,
};
const expected = process.argv[2] ?? rootPackage.version;
const drift = Object.entries(versions).filter(([, version]) => version !== expected);

if (drift.length > 0) {
  console.error(`Version drift detected; expected ${expected}.`);
  for (const [source, version] of drift) console.error(`${source}: ${version ?? "missing"}`);
  process.exit(1);
}

console.log(`CloudCheck version ${expected} is consistent across release manifests.`);
