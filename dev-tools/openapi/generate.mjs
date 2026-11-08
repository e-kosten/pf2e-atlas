import { spawnSync } from "node:child_process";
import {
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const schema = join(root, "crates/atlas-web/openapi.json");
const client = join(root, "web/atlas-ui/src/generated/http.ts");
const models = join(root, "web/atlas-ui/src/generated/models.ts");
const scratch = mkdtempSync(join(tmpdir(), "atlas-openapi-"));
const draftSchema = join(scratch, "openapi.json");
// The generated import of the Fetch hook depends on the client's directory.
const draftClient = join(
  root,
  "web/atlas-ui/src/generated",
  `.http-${process.pid}.ts`,
);
const draftModels = join(scratch, "models.ts");
const check = process.argv.includes("--check");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with ${result.status ?? result.signal}`);
  }
  return result.stdout;
}

try {
  const document = run("cargo", [
    "run",
    "--locked",
    "--quiet",
    "-p",
    "atlas-web",
    "--no-default-features",
    "--example",
    "export_openapi",
  ]);
  writeFileSync(draftSchema, document);
  const cli = fileURLToPath(
    new URL("node_modules/orval/dist/bin/orval.mjs", import.meta.url),
  );
  run(
    process.execPath,
    [
      cli,
      "--config",
      fileURLToPath(new URL("orval.config.mjs", import.meta.url)),
    ],
    {
      env: {
        ...process.env,
        ATLAS_OPENAPI_SCHEMA: draftSchema,
        ATLAS_OPENAPI_CLIENT: draftClient,
        ATLAS_OPENAPI_MODELS: draftModels,
      },
      stdio: "inherit",
    },
  );
  // The schema-only target leaves an extra blank line after its last declaration.
  writeFileSync(
    draftModels,
    readFileSync(draftModels, "utf8").trimEnd() + "\n",
  );
  if (check) {
    for (const [tracked, draft] of [
      [schema, draftSchema],
      [client, draftClient],
      [models, draftModels],
    ]) {
      if (
        readFileSync(tracked, "utf8").replaceAll("\r\n", "\n") !==
        readFileSync(draft, "utf8")
      ) {
        throw new Error(
          `${tracked} is stale. Run npm --prefix dev-tools/openapi run generate.`,
        );
      }
    }
  } else {
    writeFileSync(schema, document);
    renameSync(draftClient, client);
    writeFileSync(models, readFileSync(draftModels));
  }
} finally {
  rmSync(draftClient, { force: true });
  rmSync(scratch, { recursive: true, force: true });
}
