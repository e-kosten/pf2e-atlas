import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  existsSync,
  lstatSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, relative, resolve, sep } from "node:path";
import { env, pid } from "node:process";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

const metadataPath = "crates/atlas-web/api-contract.json";
const receiptName = "build-provenance.json";

function contractMetadata(root: string) {
  const manifest = resolve(root, metadataPath);
  const metadata = JSON.parse(readFileSync(manifest, "utf8")) as {
    protocol_revision?: unknown;
    bindings?: unknown;
  };
  const revision = metadata?.protocol_revision;
  if (
    typeof revision !== "number" ||
    !Number.isInteger(revision) ||
    revision < 1 ||
    revision > 0xffffffff
  ) {
    throw new Error("API protocol_revision must be a positive 32-bit integer");
  }
  if (typeof metadata.bindings !== "string" || !metadata.bindings) {
    throw new Error("API bindings must name the generated binding directory");
  }
  return { manifest, revision, bindingRoot: resolve(root, metadata.bindings) };
}

// Keep framing in sync with atlas-web/build_contract.rs; both test one shared vector.
export function readAtlasApiContract(root: string) {
  const { manifest, revision, bindingRoot } = contractMetadata(root);
  const files: string[] = [];
  function collect(path: string) {
    if (statSync(path).isDirectory()) {
      for (const entry of readdirSync(path)) collect(resolve(path, entry));
    } else if (path.endsWith(".ts")) {
      files.push(relative(root, path).split(sep).join("/"));
    }
  }
  collect(bindingRoot);
  if (!files.length) {
    throw new Error("API binding directory contains no generated TypeScript contracts");
  }
  const hash = createHash("sha256")
    .update("atlas-web-protocol\0")
    .update(String(revision))
    .update("\0");
  for (const file of files.sort()) {
    hash.update(file).update("\0");
    hash.update(readFileSync(resolve(root, file), "utf8").replaceAll("\r\n", "\n"));
    hash.update("\0");
  }
  return {
    identifier: `atlas-web-v2-${hash.digest("hex")}`,
    watchedPaths: [manifest, bindingRoot],
  };
}

function productionUiInput(name: string) {
  const prefix = "web/atlas-ui/";
  if (!name.startsWith(prefix)) return false;
  const local = name.slice(prefix.length);
  return (
    !/^(e2e|docs|tests|artwork|dist|node_modules|coverage|playwright-report|test-results)\//.test(
      local,
    ) &&
    !/(^|\/)(__tests__|test)\//.test(local) &&
    !/\.(test|spec)\.[cm]?[jt]sx?$/.test(local) &&
    !/(^|\/)(vitest|playwright|eslint)(\.[^/]+)?\.config\.[^/]+$/.test(local) &&
    local !== "tsconfig.tools.json" &&
    !/\.md$/i.test(local) &&
    !/(^|\/)(\.prettier[^/]*|\.gitignore|\.editorconfig)$/.test(local) &&
    !/(^|\/)\.env(\.[^/]+)?$/.test(local) &&
    !/\.tsbuildinfo$/.test(local)
  );
}

// Build identity covers effective inputs, not HEAD: staging or committing the
// same files cannot make a valid bundle stale. Deletions change the file set.
export function frontendInputIdentity(root: string): string {
  const { bindingRoot } = contractMetadata(root);
  const bindingPrefix = relative(root, bindingRoot).split(sep).join("/");
  const listed = execFileSync(
    "git",
    ["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
    { cwd: root, encoding: "utf8" },
  ).split("\0");
  const names = new Set(
    listed.filter(
      (name) =>
        name &&
        (productionUiInput(name) ||
          name === metadataPath ||
          (name.startsWith(bindingPrefix + "/") && name.endsWith(".ts"))),
    ),
  );
  // Vite reads these even when Git ignores developer-local environment files.
  for (const file of [
    ".env",
    ".env.local",
    ".env.production",
    ".env.production.local",
  ]) {
    const name = `web/atlas-ui/${file}`;
    if (existsSync(resolve(root, name))) names.add(name);
  }
  const hash = createHash("sha256").update("atlas-ui-inputs-v1\0");
  for (const name of [...names].sort()) {
    const path = resolve(root, name);
    let info;
    try {
      info = lstatSync(path);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") continue;
      throw error;
    }
    if (!statSync(path).isFile()) {
      throw new Error(`Frontend input is not a regular file: ${name}`);
    }
    const mode = info.isSymbolicLink()
      ? "120000"
      : info.mode & 0o111
        ? "100755"
        : "100644";
    hash.update(name).update("\0").update(mode).update("\0");
    if (info.isSymbolicLink()) hash.update(readlinkSync(path)).update("\0");
    hash.update(readFileSync(path)).update("\0");
  }
  hash
    .update("NODE_ENV\0")
    .update(env.NODE_ENV ?? "production")
    .update("\0");
  for (const name of Object.keys(env)
    .filter((name) => name.startsWith("VITE_"))
    .sort()) {
    hash
      .update(name)
      .update("\0")
      .update(env[name] ?? "")
      .update("\0");
  }
  return hash.digest("hex");
}

export interface FrontendBuildReceipt {
  version: 1;
  input_sha256: string;
  api_contract: string;
  sha256: Record<string, string>;
}

function bundleDigests(directory: string): Record<string, string> {
  const files = Object.create(null) as Record<string, string>;
  function visit(local: string) {
    for (const entry of readdirSync(resolve(directory, local), {
      withFileTypes: true,
    }).sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))) {
      const name = local ? `${local}/${entry.name}` : entry.name;
      if (name === receiptName) continue;
      if (entry.isDirectory()) visit(name);
      else if (entry.isFile()) {
        files[name] = createHash("sha256")
          .update(readFileSync(resolve(directory, name)))
          .digest("hex");
      } else throw new Error(`Frontend bundle output is not a regular file: ${name}`);
    }
  }
  visit("");
  if (!files["index.html"] || !files["api-contract.txt"]) {
    throw new Error(
      "Frontend bundle is missing index.html or api-contract.txt; rebuild it",
    );
  }
  return files;
}

export function verifyFrontendBuild(
  root: string,
  dist = resolve(root, "web/atlas-ui/dist"),
): FrontendBuildReceipt {
  const directory = resolve(root, dist);
  const receipt: FrontendBuildReceipt = JSON.parse(
    readFileSync(resolve(directory, receiptName), "utf8"),
  );
  if (
    receipt?.version !== 1 ||
    typeof receipt.input_sha256 !== "string" ||
    typeof receipt.api_contract !== "string" ||
    !receipt.sha256 ||
    Array.isArray(receipt.sha256) ||
    typeof receipt.sha256 !== "object" ||
    Object.values(receipt.sha256).some((value) => !/^[a-f0-9]{64}$/.test(value))
  ) {
    throw new Error("Frontend build receipt is invalid; rebuild the frontend");
  }
  const inputs = frontendInputIdentity(root);
  if (receipt.input_sha256 !== inputs) {
    throw new Error(
      "Frontend source inputs changed since the build; rebuild the frontend",
    );
  }
  const contract = readAtlasApiContract(root).identifier;
  if (
    receipt.api_contract !== contract ||
    readFileSync(resolve(directory, "api-contract.txt"), "utf8").trim() !== contract
  ) {
    throw new Error("Frontend build has another API contract; rebuild the frontend");
  }
  const actual = bundleDigests(directory);
  const names = Object.keys(actual).sort();
  if (
    JSON.stringify(names) !== JSON.stringify(Object.keys(receipt.sha256).sort()) ||
    names.some((name) => actual[name] !== receipt.sha256[name])
  ) {
    throw new Error(
      "Frontend bundle files differ from their build receipt; rebuild the frontend",
    );
  }
  if (frontendInputIdentity(root) !== inputs) {
    throw new Error(
      "Frontend inputs changed during verification; retry after edits finish",
    );
  }
  return receipt;
}

export function atlasApiContract(
  root = resolve(dirname(fileURLToPath(import.meta.url)), "../.."),
): Plugin {
  const { identifier: contract, watchedPaths } = readAtlasApiContract(root);
  let buildInputs: string | undefined;
  let mode = "production";
  return {
    name: "atlas-api-contract",
    config: () => ({ define: { __ATLAS_API_CONTRACT__: JSON.stringify(contract) } }),
    configResolved(config) {
      mode = config.mode;
    },
    buildStart() {
      buildInputs = frontendInputIdentity(root);
    },
    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "api-contract.txt",
        source: contract + "\n",
      });
    },
    writeBundle: {
      order: "post",
      handler(options) {
        const directory = resolve(root, "web/atlas-ui", options.dir ?? "dist");
        const receipt = resolve(directory, receiptName);
        rmSync(receipt, { force: true });
        if (mode !== "production") {
          throw new Error("Authenticated frontend builds require production mode");
        }
        const outputs = bundleDigests(directory);
        if (
          !buildInputs ||
          frontendInputIdentity(root) !== buildInputs ||
          readAtlasApiContract(root).identifier !== contract
        ) {
          throw new Error(
            "Frontend inputs changed during the build; finish edits and rebuild",
          );
        }
        const result: FrontendBuildReceipt = {
          version: 1,
          input_sha256: buildInputs,
          api_contract: contract,
          sha256: outputs,
        };
        const temporary = `${receipt}.${pid}.tmp`;
        writeFileSync(temporary, JSON.stringify(result, null, 2) + "\n");
        renameSync(temporary, receipt);
      },
    },
    configureServer(server) {
      server.watcher.add(watchedPaths);
      const restart = (path: string) => {
        if (
          watchedPaths.some((input) => path === input || path.startsWith(input + sep))
        ) {
          void server.restart();
        }
      };
      server.watcher.on("change", restart).on("add", restart).on("unlink", restart);
    },
  };
}
