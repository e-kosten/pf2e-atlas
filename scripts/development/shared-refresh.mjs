import {
  existsSync,
  copyFileSync,
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { buildProfile, selectedProfile } from "./build-profile.mjs";
import { runChecks } from "../validation/run-check.mjs";
import {
  digest,
  output,
  requireReadiness,
  retainedTool,
  toolCapabilities,
} from "./preflight.mjs";

const root = output("git", ["rev-parse", "--show-toplevel"], process.cwd());
let temporary;
try {
  const profile = buildProfile(
    root,
    selectedProfile(process.argv.slice(2), "release"),
  );
  const buildHint = `Run just dev-build --profile ${profile.name} on the clean, validated candidate.`;
  await requireReadiness(root, { shared: true });
  if (output("git", ["status", "--porcelain", "--untracked-files=all"], root))
    throw new Error(
      "Commit source changes before shared refresh; existing task databases are never used as staging data.",
    );
  if (!existsSync(profile.receipt))
    throw new Error(
      `Selected ${profile.name} build receipt missing. ${buildHint}`,
    );
  const receipt = JSON.parse(readFileSync(profile.receipt, "utf8"));
  const binary = profile.binary;
  if (
    receipt.version !== 1 ||
    receipt.profile !== profile.name ||
    receipt.source?.dirty !== false ||
    receipt.source.commit !== output("git", ["rev-parse", "HEAD"], root) ||
    receipt.binary_sha256 !== digest(readFileSync(binary))
  )
    throw new Error(
      `Current clean ${profile.name} build provenance required. ${buildHint}`,
    );
  const tool = retainedTool(root);
  toolCapabilities(tool, root);
  const toolReceipt = JSON.parse(readFileSync(tool + ".json", "utf8"));
  if (toolReceipt.binary_sha256 !== digest(readFileSync(tool)))
    throw new Error("Retained tool/receipt mismatch; run just dev-tool-build.");
  const common = output(
    "git",
    ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    root,
  );
  const store = resolve(dirname(common), ".atlas/dev-baselines");
  const selected = JSON.parse(
    readFileSync(resolve(store, "current.json"), "utf8"),
  ).baseline;
  if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(selected))
    throw new Error("Invalid baseline identity");
  const seed = JSON.parse(
    readFileSync(resolve(store, "versions", selected, "manifest.json"), "utf8"),
  );
  const source = seed.assets.source,
    models = seed.assets.embedding_cache;
  if (!source || !models || !existsSync(source) || !existsSync(models))
    throw new Error(
      "Shared source/model paths must exist; no assets are downloaded automatically.",
    );
  temporary = mkdtempSync(resolve(tmpdir(), "atlas-shared-refresh-"));
  const frozenBinary = resolve(
    temporary,
    process.platform === "win32" ? "atlas.exe" : "atlas",
  );
  const frozenTool = resolve(
    temporary,
    process.platform === "win32" ? "atlas-dev.exe" : "atlas-dev",
  );
  copyFileSync(binary, frozenBinary);
  copyFileSync(tool, frozenTool);
  chmodSync(frozenBinary, 0o555);
  chmodSync(frozenTool, 0o555);
  if (
    digest(readFileSync(frozenBinary)) !== receipt.binary_sha256 ||
    digest(readFileSync(frozenTool)) !== toolReceipt.binary_sha256
  )
    throw new Error(
      "Maintenance executable changed while freezing; shared baseline preserved.",
    );
  toolCapabilities(frozenTool, root);
  const checkout = resolve(temporary, "checkout");
  const index = resolve(checkout, ".atlas/workspace.sqlite");
  const commit = receipt.source.commit;
  const prepared = await runChecks([
    [
      "private seed checkout",
      "git",
      "clone",
      "--quiet",
      "--shared",
      "--no-checkout",
      root,
      checkout,
    ],
    [
      "pin seed checkout",
      "git",
      "-C",
      checkout,
      "checkout",
      "--quiet",
      "--detach",
      commit,
    ],
  ]);
  if (prepared) {
    process.exitCode = prepared;
  } else {
    process.env.ATLAS_VALIDATION_OUTPUT_ROOT = resolve(
      root,
      process.env.ATLAS_VALIDATION_OUTPUT_ROOT || ".atlas/validation",
    );
    process.chdir(checkout);
    console.error(
      "Maintenance candidate frozen. Source edits may resume; keep the caller checkout present until publication finishes.",
    );
    const code = await runChecks([
      [
        "stage controlled seed",
        frozenTool,
        "baseline",
        "init",
        "--checkout",
        checkout,
        "--store",
        store,
        "--baseline",
        selected,
        "--json",
      ],
      [
        "upgrade controlled seed",
        frozenBinary,
        "setup",
        "--offline",
        "--path-mode",
        "global",
        "--index",
        index,
        "--source",
        source,
        "--embedding-cache-path",
        models,
        "--json",
        "--progress",
        "always",
      ],
      [
        "publish validated shared seed",
        frozenTool,
        "baseline",
        "publish",
        "--checkout",
        root,
        "--store",
        store,
        "--binary",
        frozenBinary,
        "--index",
        index,
        "--build-commit",
        commit,
        "--build-profile",
        profile.name,
        "--toolchain",
        receipt.toolchain,
        "--source",
        source,
        "--embedding-cache-path",
        models,
        "--json",
        "--progress",
        "always",
      ],
    ]);
    process.exitCode = code;
    if (!code)
      console.error(
        "Shared seed refreshed. Existing task receipts/databases remain pinned and unchanged.",
      );
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
} finally {
  process.chdir(root);
  if (temporary) rmSync(temporary, { recursive: true, force: true });
}
