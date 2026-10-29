import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

test("browser artifacts reject stale source, corrupt contents and incompatible hosts", async () => {
  const root = mkdtempSync(resolve(tmpdir(), "atlas-browser-artifact-"));
  const git = (...args) =>
    execFileSync("git", args, { cwd: root, stdio: "pipe" });
  try {
    mkdirSync(resolve(root, "scripts"));
    copyFileSync(
      new URL("./browser-fixture.mjs", import.meta.url),
      resolve(root, "scripts/browser-fixture.mjs"),
    );
    mkdirSync(resolve(root, "web/atlas-ui/src"), { recursive: true });
    mkdirSync(resolve(root, "crates/atlas-web"), { recursive: true });
    mkdirSync(resolve(root, "crates/atlas-app-model/bindings"), {
      recursive: true,
    });
    copyFileSync(
      new URL("../web/atlas-ui/apiContract.ts", import.meta.url),
      resolve(root, "web/atlas-ui/apiContract.ts"),
    );
    writeFileSync(
      resolve(root, "crates/atlas-web/api-contract.json"),
      JSON.stringify({
        protocol_revision: 1,
        bindings: "crates/atlas-app-model/bindings",
      }),
    );
    writeFileSync(
      resolve(root, "crates/atlas-app-model/bindings/Test.ts"),
      "export type Test = string;\n",
    );
    writeFileSync(
      resolve(root, "web/atlas-ui/src/current.ts"),
      "export const current = 1;\n",
    );
    git("init", "-q");
    git("config", "user.name", "Atlas fixture");
    git("config", "user.email", "atlas-fixture@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", "/dev/null");
    writeFileSync(
      resolve(root, ".gitignore"),
      "target/\nweb/atlas-ui/dist/\nartifact*/\nbin/\nbuild-order\n",
    );
    git("add", ".");
    git("commit", "-qm", "test: candidate");
    mkdirSync(resolve(root, "target/debug/examples"), { recursive: true });
    mkdirSync(resolve(root, "web/atlas-ui/dist"), { recursive: true });
    writeFileSync(
      resolve(root, "target/debug/examples/browser_fixture"),
      "#!/bin/sh\nexit 0\n",
    );
    writeFileSync(
      resolve(root, "web/atlas-ui/dist/index.html"),
      "candidate frontend",
    );
    const { frontendInputIdentity, readAtlasApiContract } = await import(
      pathToFileURL(resolve(root, "web/atlas-ui/apiContract.ts"))
    );
    const contract = readAtlasApiContract(root).identifier;
    writeFileSync(
      resolve(root, "web/atlas-ui/dist/api-contract.txt"),
      contract + "\n",
    );
    writeFileSync(
      resolve(root, "web/atlas-ui/dist/build-provenance.json"),
      JSON.stringify({
        version: 1,
        input_sha256: frontendInputIdentity(root),
        api_contract: contract,
        sha256: {
          "index.html": createHash("sha256")
            .update("candidate frontend")
            .digest("hex"),
          "api-contract.txt": createHash("sha256")
            .update(contract + "\n")
            .digest("hex"),
        },
      }),
    );
    mkdirSync(resolve(root, "bin"));
    for (const tool of ["cargo", "strip"]) {
      const path = resolve(root, "bin", tool);
      writeFileSync(
        path,
        `#!/bin/sh\nprintf '${tool} %s\\n' "$*" >> build-order\n`,
      );
      chmodSync(path, 0o755);
    }
    const environment = {
      ...process.env,
      PATH: `${resolve(root, "bin")}:${process.env.PATH}`,
    };
    const script = resolve(root, "scripts/browser-fixture.mjs");
    const artifact = resolve(root, "artifact with ' quote");
    execFileSync(process.execPath, [script, artifact], { env: environment });
    assert.equal(
      readFileSync(resolve(root, "build-order"), "utf8"),
      "cargo build --locked -p atlas-web --example browser_fixture\n" +
        (process.platform === "linux"
          ? "strip --strip-debug target/debug/examples/browser_fixture\n"
          : ""),
    );
    const { browserFixture } = await import(pathToFileURL(script));
    const selected = browserFixture(artifact);
    assert.equal(readFileSync(selected.index, "utf8"), "candidate frontend");
    execFileSync("sh", ["-c", selected.command]);
    assert.equal(
      readFileSync(browserFixture("").index, "utf8"),
      "candidate frontend",
    );
    assert.equal(
      browserFixture("").command,
      "cargo run --locked -p atlas-web --example browser_fixture --",
    );
    for (const file of ["browser_fixture", "index.html"]) {
      const path = resolve(artifact, file);
      const original = readFileSync(path);
      writeFileSync(path, "wrong candidate");
      assert.throws(() => browserFixture(artifact), /checksum/);
      writeFileSync(path, original);
    }
    const path = resolve(artifact, "manifest.json");
    const manifest = JSON.parse(readFileSync(path));
    for (const key of ["commit", "platform", "arch"]) {
      writeFileSync(path, JSON.stringify({ ...manifest, [key]: "wrong" }));
      assert.throws(() => browserFixture(artifact), /another/);
    }
    writeFileSync(path, JSON.stringify(manifest));
    writeFileSync(
      resolve(root, "scripts/untracked-test.mjs"),
      "uncommitted source\n",
    );
    assert.throws(
      () =>
        execFileSync(
          process.execPath,
          [script, resolve(root, "artifact-untracked")],
          { env: environment, stdio: "pipe" },
        ),
      /Commit source changes/,
    );
    rmSync(resolve(root, "scripts/untracked-test.mjs"));
    const cargo = resolve(root, "bin/cargo");
    const originalCargo = readFileSync(cargo);
    writeFileSync(
      cargo,
      "#!/bin/sh\nprintf '\\n' >> scripts/browser-fixture.mjs\n",
    );
    assert.throws(
      () =>
        execFileSync(
          process.execPath,
          [script, resolve(root, "artifact-mutated")],
          { env: environment, stdio: "pipe" },
        ),
      /Candidate changed while building/,
    );
    git("checkout", "--", "scripts/browser-fixture.mjs");
    writeFileSync(cargo, originalCargo);
    const copyHook = resolve(root, "bin/copy-hook.cjs");
    writeFileSync(
      copyHook,
      `const fs=require('node:fs');const path=require('node:path');const original=fs.copyFileSync;
fs.copyFileSync=function(from,to,...options){
 const result=original(from,to,...options);
 if(path.basename(from)==='browser_fixture'&&path.basename(path.dirname(to))==='artifact-copy-race')fs.appendFileSync(path.join(process.env.ATLAS_TEST_COPY_ROOT,'scripts/browser-fixture.mjs'),'\\n');
 return result;
};require('node:module').syncBuiltinESMExports();`,
    );
    assert.throws(
      () =>
        execFileSync(
          process.execPath,
          [script, resolve(root, "artifact-copy-race")],
          {
            env: {
              ...environment,
              NODE_OPTIONS: `--require=${copyHook}`,
              ATLAS_TEST_COPY_ROOT: root,
            },
            stdio: "pipe",
          },
        ),
      /Candidate changed while building/,
    );
    git("checkout", "--", "scripts/browser-fixture.mjs");
    writeFileSync(
      resolve(root, "scripts/browser-fixture.mjs"),
      readFileSync(script, "utf8") + "\n",
    );
    assert.throws(
      () =>
        execFileSync(
          process.execPath,
          [script, resolve(root, "artifact-dirty")],
          { stdio: "pipe" },
        ),
      /Commit source changes/,
    );
    git("checkout", "--", "scripts/browser-fixture.mjs");
    const wireBefore = readAtlasApiContract(root).identifier;
    writeFileSync(
      resolve(root, "web/atlas-ui/src/current.ts"),
      "export const current = 2;\n",
    );
    git("add", ".");
    git("commit", "-qm", "test: UI changed without a wire change");
    assert.equal(readAtlasApiContract(root).identifier, wireBefore);
    assert.throws(
      () =>
        execFileSync(
          process.execPath,
          [script, resolve(root, "artifact-stale-ui")],
          { env: environment, stdio: "pipe" },
        ),
      /[Ss]tale|[Ff]rontend.*inputs|[Ff]rontend.*build/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
