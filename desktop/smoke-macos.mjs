import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
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

for (const mode of ["normal", "retry", "preflight"]) {
  run(mode);
}

function run(mode) {
  const home = mkdtempSync(join(realpathSync(tmpdir()), "atlas-native-smoke-"));
  const source = join(home, "source");

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
    const result = spawnSync(join(bundle, "Contents/MacOS/atlas-desktop"), {
      cwd: home,
      env: {
        ...process.env,
        HOME: home,
        PATH: "/nonexistent",
        ATLAS_PILOT_HOME: home,
        ATLAS_PILOT_SMOKE: mode,
      },
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
