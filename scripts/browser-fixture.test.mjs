import { test } from "node:test";
import assert from "node:assert/strict";
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
    git("init", "-q");
    git("config", "user.name", "Atlas fixture");
    git("config", "user.email", "atlas-fixture@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", "/dev/null");
    writeFileSync(resolve(root, ".gitignore"), "target/\nweb/\nartifact*/\n");
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
    mkdirSync(resolve(root, "bin"));
    for (const tool of ["cargo", "strip"]) {
      const path = resolve(root, "bin", tool);
      writeFileSync(path, `#!/bin/sh\nprintf '${tool}\\n' >> build-order\n`);
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
      process.platform === "linux" ? "cargo\nstrip\n" : "cargo\n",
    );
    const { browserArtifact } = await import(pathToFileURL(script));
    const selected = browserArtifact(artifact);
    assert.equal(readFileSync(selected.index, "utf8"), "candidate frontend");
    execFileSync("sh", ["-c", selected.command]);
    assert.equal(browserArtifact(""), undefined);
    for (const file of ["browser_fixture", "index.html"]) {
      const path = resolve(artifact, file);
      const original = readFileSync(path);
      writeFileSync(path, "wrong candidate");
      assert.throws(() => browserArtifact(artifact), /checksum/);
      writeFileSync(path, original);
    }
    const path = resolve(artifact, "manifest.json");
    const manifest = JSON.parse(readFileSync(path));
    for (const key of ["commit", "platform", "arch"]) {
      writeFileSync(path, JSON.stringify({ ...manifest, [key]: "wrong" }));
      assert.throws(() => browserArtifact(artifact), /another/);
    }
    writeFileSync(path, JSON.stringify(manifest));
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
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
