import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "darwin") {
  throw new Error("Run the native smoke test in a macOS desktop session.");
}

const defaultBundle = fileURLToPath(
  new URL("../target/debug/bundle/macos/PF2e Atlas.app", import.meta.url),
);
const bundle = resolve(process.argv[2] ?? defaultBundle);

const pin = JSON.parse(readFileSync(new URL("../dev-tools/source-contracts/source-pin.json", import.meta.url))).commit;
const modes = ["normal", "retry", "preflight", "reference", "selector"];
const selected = process.argv[3];
if (selected && !modes.includes(selected)) throw new Error("Unknown native smoke scenario");
for (const mode of selected ? [selected] : modes) {
  run(mode);
}

function run(mode) {
  const home = mkdtempSync(join(realpathSync(tmpdir()), "atlas-native-smoke-"));
  const source = mode === "reference"
    ? join(home, "Library/Caches/pf2e-atlas/pf2e", pin)
    : join(home, "source");
  const env = {
    ...process.env,
    HOME: home,
    PATH: "/nonexistent",
    ATLAS_PILOT_HOME: home,
    ATLAS_PILOT_SMOKE: mode,
  };

  function writeFixture(path, value) {
    writeFileSync(join(source, path), JSON.stringify(value));
  }

  try {
    mkdirSync(join(source, "packs/actions"), { recursive: true });
    mkdirSync(join(source, "static/lang"), { recursive: true });
    writeFixture("module.json", {
      packs: [{ name: "actions", label: "Actions", type: "Item", path: "packs/actions" }],
    });
    writeFixture("static/lang/en.json", {});
    writeFixture("packs/actions/handoff.json", {
      _id: "aaaaaaaaaaaaaaaa",
      name: "Handoff Test Action",
      type: "action",
      system: {
        description: { value: "A fixture for the native setup handoff." },
        traits: { value: [] },
      },
    });
    if (mode === "reference" || mode === "selector") {
      const seeded = spawnSync(join(bundle, "Contents/MacOS/atlas"), [
        "setup", "--offline", "--no-embeddings", "--json",
        ...(mode === "selector" ? ["--source", source] : []),
      ], { cwd: home, env, encoding: "utf8", timeout: 8_000 });
      if (seeded.status !== 0) {
        throw new Error(`Could not seed the reference refresh fixture: ${seeded.stdout}${seeded.stderr}`);
      }
      if (mode === "reference") writeFixture(".hold-reference", true);
    }
    const result = spawnSync(join(bundle, "Contents/MacOS/atlas-desktop"), {
      cwd: home,
      env,
      encoding: "utf8",
      timeout: 15_000,
      killSignal: "SIGKILL",
    });
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    if (result.error) throw result.error;
    if (result.status !== 0 || !result.stdout.includes("Atlas native smoke passed")) {
      throw new Error("The native smoke test requires a debug bundle and a working macOS desktop session.");
    }
  } finally {
    rmSync(home, { recursive: true, force: true });
  }
}
