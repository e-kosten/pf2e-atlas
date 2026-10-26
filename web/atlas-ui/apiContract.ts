import { createHash } from "node:crypto";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite";

export function atlasApiContract(): Plugin {
  const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const manifest = resolve(root, "crates/atlas-web/api-contract-inputs.json");
  const inputs: string[] = JSON.parse(readFileSync(manifest, "utf8"));
  const files: string[] = [];
  function collect(path: string) {
    if (statSync(path).isDirectory()) {
      for (const entry of readdirSync(path)) collect(resolve(path, entry));
    } else if (/\.(rs|ts)$/.test(path)) {
      files.push(relative(root, path).split(sep).join("/"));
    }
  }
  for (const input of inputs) collect(resolve(root, input));
  const hash = createHash("sha256");
  for (const file of files.sort()) {
    hash.update(file).update("\0");
    hash.update(readFileSync(resolve(root, file), "utf8").replaceAll("\r\n", "\n"));
    hash.update("\0");
  }
  const contract = `atlas-web-v1-${hash.digest("hex")}`;
  return {
    name: "atlas-api-contract",
    config: () => ({ define: { __ATLAS_API_CONTRACT__: JSON.stringify(contract) } }),
    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "api-contract.txt",
        source: contract + "\n",
      });
    },
    configureServer(server) {
      const paths = [manifest, ...inputs.map((input) => resolve(root, input))];
      server.watcher.add(paths);
      const restart = (path: string) => {
        if (paths.some((input) => path === input || path.startsWith(input + sep))) {
          void server.restart();
        }
      };
      server.watcher.on("change", restart).on("add", restart).on("unlink", restart);
    },
  };
}
