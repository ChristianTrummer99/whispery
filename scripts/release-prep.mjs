import fs from "node:fs/promises";
import path from "node:path";

const VERSION_REGEX = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

function usage() {
  console.error("Usage: npm run release:prep -- <version> [--dry-run]");
  process.exit(1);
}

function parseArgs(argv) {
  const args = argv.slice(2);
  const dryRun = args.includes("--dry-run");
  const version = args.find((arg) => !arg.startsWith("--"));

  if (!version) usage();
  if (!VERSION_REGEX.test(version)) {
    console.error(`Invalid version "${version}". Expected semver like 0.1.2`);
    process.exit(1);
  }

  return { version, dryRun };
}

async function updatePackageJson(rootDir, version, dryRun) {
  const file = path.join(rootDir, "package.json");
  const raw = await fs.readFile(file, "utf8");
  const json = JSON.parse(raw);
  const previous = json.version;
  json.version = version;

  if (!dryRun) {
    await fs.writeFile(file, `${JSON.stringify(json, null, 2)}\n`, "utf8");
  }

  return { file, previous, next: version };
}

async function updateTauriConfig(rootDir, version, dryRun) {
  const file = path.join(rootDir, "src-tauri", "tauri.conf.json");
  const raw = await fs.readFile(file, "utf8");
  const json = JSON.parse(raw);
  const previous = json.version;
  json.version = version;

  if (!dryRun) {
    await fs.writeFile(file, `${JSON.stringify(json, null, 2)}\n`, "utf8");
  }

  return { file, previous, next: version };
}

async function updateCargoToml(rootDir, version, dryRun) {
  const file = path.join(rootDir, "src-tauri", "Cargo.toml");
  const raw = await fs.readFile(file, "utf8");
  const match = raw.match(/^\s*version\s*=\s*"([^"]+)"/m);

  if (!match) {
    throw new Error("Could not find [package] version in src-tauri/Cargo.toml");
  }

  const previous = match[1];
  const updated = raw.replace(
    /^\s*version\s*=\s*"([^"]+)"/m,
    `version = "${version}"`
  );

  if (!dryRun) {
    await fs.writeFile(file, updated, "utf8");
  }

  return { file, previous, next: version };
}

async function main() {
  const { version, dryRun } = parseArgs(process.argv);
  const rootDir = process.cwd();

  const updates = await Promise.all([
    updatePackageJson(rootDir, version, dryRun),
    updateTauriConfig(rootDir, version, dryRun),
    updateCargoToml(rootDir, version, dryRun),
  ]);

  console.log(dryRun ? "Dry run: files unchanged." : "Updated release versions:");
  for (const update of updates) {
    console.log(`${update.file}: ${update.previous} -> ${update.next}`);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
