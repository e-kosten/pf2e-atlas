import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { env, platform } from "node:process";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
  atlasApiContract,
  frontendInputIdentity,
  readAtlasApiContract,
  verifyFrontendBuild,
} from "./apiContract";

describe("browser wire compatibility", () => {
  let root: string;
  const metadata = "crates/atlas-web/api-contract.json";
  const write = (file: string, contents: string) => {
    const path = join(root, file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, contents);
  };

  beforeEach(() => {
    root = mkdtempSync(join(tmpdir(), "atlas-api-contract-"));
    write(metadata, JSON.stringify({ protocol_revision: 7, bindings: "bindings" }));
    // Reverse insertion order and mixed line endings match the Rust fixture.
    write("bindings/B.ts", 'export type B = "ready" | "busy";\n');
    write("bindings/A.ts", "export type A = { value: string };\r\n");
  });

  afterEach(() => rmSync(root, { recursive: true, force: true }));

  it("matches the shared Rust vector and normalizes line endings", () => {
    const before = readAtlasApiContract(root).identifier;
    expect(before).toBe(
      "atlas-web-v2-16d06f503005aaab173401cb8ced6199baf44f44398b368eb96407cf49dedf01",
    );
    write("bindings/A.ts", "export type A = { value: string };\n");
    expect(readAtlasApiContract(root).identifier).toBe(before);
  });

  it("ignores internal source, tests and comments and does not watch them", () => {
    const before = readAtlasApiContract(root).identifier;
    for (const file of [
      "crates/atlas-app-service/src/rules.rs",
      "crates/atlas-domain/src/similarity.rs",
      "crates/atlas-web/src/tests.rs",
    ]) {
      write(file, "// Internal implementation, test or comment change.\n");
    }
    const after = readAtlasApiContract(root);
    expect(after.identifier).toBe(before);
    expect(after.watchedPaths).toEqual([join(root, metadata), join(root, "bindings")]);
  });

  it("changes when bindings change, appear or disappear", () => {
    const before = readAtlasApiContract(root).identifier;
    write("bindings/A.ts", "export type A = { value: bigint };\n");
    const changed = readAtlasApiContract(root).identifier;
    expect(changed).not.toBe(before);
    write("bindings/C.ts", "export type C = A;\n");
    const added = readAtlasApiContract(root).identifier;
    expect(added).not.toBe(changed);
    rmSync(join(root, "bindings/A.ts"));
    expect(readAtlasApiContract(root).identifier).not.toBe(added);
  });

  it("uses the protocol revision for wire behavior absent from TypeScript", () => {
    const before = readAtlasApiContract(root).identifier;
    write(metadata, JSON.stringify({ protocol_revision: 8, bindings: "bindings" }));
    expect(readAtlasApiContract(root).identifier).not.toBe(before);
  });

  it("ignores metadata formatting and accepts equivalent numeric JSON", () => {
    const before = readAtlasApiContract(root).identifier;
    write(metadata, '{ "bindings": "bindings", "protocol_revision": 7.0 }\n');
    expect(readAtlasApiContract(root).identifier).toBe(before);
  });

  it.each([0, -1, 1.5, 4294967296, null, "1"])(
    "rejects invalid protocol revision %s",
    (protocol_revision) => {
      write(metadata, JSON.stringify({ protocol_revision, bindings: "bindings" }));
      expect(() => readAtlasApiContract(root)).toThrow("protocol_revision");
    },
  );

  it("rejects an empty binding inventory", () => {
    rmSync(join(root, "bindings/A.ts"));
    rmSync(join(root, "bindings/B.ts"));
    expect(() => readAtlasApiContract(root)).toThrow("no generated TypeScript");
  });
});

describe("frontend build provenance", () => {
  let root: string;
  const dist = () => join(root, "web/atlas-ui/dist");
  const write = (file: string, contents: string) => {
    const path = join(root, file);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, contents);
  };
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, stdio: "pipe" });
  const beginBuild = () => {
    const plugin = atlasApiContract(root);
    (plugin.buildStart as () => void)();
    write("web/atlas-ui/dist/index.html", "<html>candidate A</html>\n");
    write("web/atlas-ui/dist/assets/app.js", "console.log('candidate A');\n");
    write(
      "web/atlas-ui/dist/api-contract.txt",
      readAtlasApiContract(root).identifier + "\n",
    );
    return () =>
      (plugin.writeBundle as { handler: (options: { dir: string }) => void }).handler({
        dir: dist(),
      });
  };

  beforeEach(() => {
    root = mkdtempSync(join(tmpdir(), "atlas-frontend-build-"));
    git("init", "-q");
    write(".gitignore", "web/atlas-ui/dist/\nweb/atlas-ui/.env.local\n");
    write(
      "crates/atlas-web/api-contract.json",
      JSON.stringify({
        protocol_revision: 1,
        bindings: "crates/atlas-app-model/bindings",
      }),
    );
    write(
      "crates/atlas-app-model/bindings/View.ts",
      "export type View = { value: string };\n",
    );
    write("web/atlas-ui/src/App.tsx", "export const title = 'candidate A';\n");
    write("web/atlas-ui/package-lock.json", '{"lockfileVersion":3}\n');
  });

  afterEach(() => rmSync(root, { recursive: true, force: true }));

  it("authenticates every output and survives staging unchanged inputs", () => {
    beginBuild()();
    const before = verifyFrontendBuild(root);
    expect(Object.keys(before.sha256).sort()).toEqual([
      "api-contract.txt",
      "assets/app.js",
      "index.html",
    ]);
    expect(before.sha256["assets/app.js"]).toBe(
      createHash("sha256").update("console.log('candidate A');\n").digest("hex"),
    );
    git("add", ".");
    expect(verifyFrontendBuild(root)).toEqual(before);
  });

  it("rejects a clean newer UI candidate with an unchanged API", () => {
    beginBuild()();
    const api = readAtlasApiContract(root).identifier;
    write("web/atlas-ui/src/App.tsx", "export const title = 'candidate B';\n");
    git("add", ".");
    git(
      "-c",
      "user.name=Atlas Test",
      "-c",
      "user.email=atlas@example.invalid",
      "commit",
      "-qm",
      "candidate B",
    );
    expect(readAtlasApiContract(root).identifier).toBe(api);
    expect(() => verifyFrontendBuild(root)).toThrow("source inputs changed");
  });

  it("allows internal Rust, tests, docs and browser configuration changes", () => {
    beginBuild()();
    const before = verifyFrontendBuild(root);
    for (const file of [
      "crates/atlas-app-service/src/rules.rs",
      "web/atlas-ui/src/App.test.tsx",
      "web/atlas-ui/src/test/setup.ts",
      "web/atlas-ui/e2e/flow.spec.ts",
      "web/atlas-ui/docs/frontend-guidelines.md",
      "web/atlas-ui/vitest.config.ts",
      "web/atlas-ui/playwright.devices.config.ts",
      "web/atlas-ui/tsconfig.tools.json",
    ])
      write(file, "// A change that does not build production UI.\n");
    expect(verifyFrontendBuild(root)).toEqual(before);
  });

  it.each(["modify", "delete", "extra"])("rejects %s bundle files", (operation) => {
    beginBuild()();
    if (operation === "delete") rmSync(join(dist(), "assets/app.js"));
    else
      write(
        `web/atlas-ui/dist/${operation === "extra" ? "__proto__" : "assets/app.js"}`,
        "tampered\n",
      );
    expect(() => verifyFrontendBuild(root)).toThrow("bundle files differ");
  });

  it.each(["missing", "invalid"])("rejects %s build receipt", (operation) => {
    beginBuild()();
    if (operation === "missing") rmSync(join(dist(), "build-provenance.json"));
    else write("web/atlas-ui/dist/build-provenance.json", '{"version":0}\n');
    expect(() => verifyFrontendBuild(root)).toThrow();
  });

  it("rejects API sidecar tampering even with a rewritten asset digest", () => {
    beginBuild()();
    const receipt = verifyFrontendBuild(root);
    write("web/atlas-ui/dist/api-contract.txt", "wrong-api\n");
    receipt.sha256["api-contract.txt"] = createHash("sha256")
      .update("wrong-api\n")
      .digest("hex");
    write("web/atlas-ui/dist/build-provenance.json", JSON.stringify(receipt));
    expect(() => verifyFrontendBuild(root)).toThrow("another API contract");
  });

  it("detects untracked production files, deletions and lockfile changes", () => {
    const before = frontendInputIdentity(root);
    write("web/atlas-ui/src/NewView.tsx", "export const value = 1;\n");
    expect(frontendInputIdentity(root)).not.toBe(before);
    rmSync(join(root, "web/atlas-ui/src/NewView.tsx"));
    expect(frontendInputIdentity(root)).toBe(before);
    git("add", ".");
    rmSync(join(root, "web/atlas-ui/src/App.tsx"));
    const deleted = frontendInputIdentity(root);
    expect(deleted).not.toBe(before);
    git("add", "-u");
    expect(frontendInputIdentity(root)).toBe(deleted);
    write("web/atlas-ui/package-lock.json", '{"lockfileVersion":3,"changed":true}\n');
    expect(frontendInputIdentity(root)).not.toBe(deleted);
  });

  it.skipIf(platform === "win32")("includes executable file modes", () => {
    const before = frontendInputIdentity(root);
    chmodSync(join(root, "web/atlas-ui/src/App.tsx"), 0o755);
    expect(frontendInputIdentity(root)).not.toBe(before);
  });

  it.skipIf(platform === "win32")(
    "includes symlink targets and their actual contents",
    () => {
      const entry = join(root, "web/atlas-ui/src/App.tsx");
      rmSync(entry);
      write("shared-one.ts", "export const value = 1;\n");
      symlinkSync("../../../shared-one.ts", entry);
      const before = frontendInputIdentity(root);
      write("shared-one.ts", "export const value = 2;\n");
      const changed = frontendInputIdentity(root);
      expect(changed).not.toBe(before);
      write("shared-two.ts", "export const value = 2;\n");
      rmSync(entry);
      symlinkSync("../../../shared-two.ts", entry);
      expect(frontendInputIdentity(root)).not.toBe(changed);
    },
  );

  it("includes ignored Vite environment files and hashes VITE values", () => {
    const before = frontendInputIdentity(root);
    write("web/atlas-ui/.env.local", "VITE_ATLAS_TEST=local\n");
    expect(frontendInputIdentity(root)).not.toBe(before);
    const key = "VITE_ATLAS_PROVENANCE_TEST";
    const previous = env[key];
    try {
      const local = frontendInputIdentity(root);
      env[key] = "changed";
      expect(frontendInputIdentity(root)).not.toBe(local);
    } finally {
      if (previous === undefined) delete env[key];
      else env[key] = previous;
    }
  });

  it("refuses a receipt if production inputs change during a build", () => {
    const finish = beginBuild();
    write("web/atlas-ui/src/App.tsx", "export const title = 'changed mid-build';\n");
    expect(finish).toThrow("changed during the build");
    expect(existsSync(join(dist(), "build-provenance.json"))).toBe(false);
  });

  it("refuses authenticated output for a nonproduction Vite mode", () => {
    const plugin = atlasApiContract(root);
    (plugin.configResolved as (config: { mode: string }) => void)({ mode: "staging" });
    (plugin.buildStart as () => void)();
    write("web/atlas-ui/dist/index.html", "staging\n");
    write(
      "web/atlas-ui/dist/api-contract.txt",
      readAtlasApiContract(root).identifier + "\n",
    );
    expect(() =>
      (plugin.writeBundle as { handler: (options: { dir: string }) => void }).handler({
        dir: dist(),
      }),
    ).toThrow("require production mode");
    expect(existsSync(join(dist(), "build-provenance.json"))).toBe(false);
  });
});
